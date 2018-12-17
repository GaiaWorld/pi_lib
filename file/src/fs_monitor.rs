use std::thread;
use std::sync::Arc;
use std::path::PathBuf;
use std::time::Duration;
use std::collections::HashMap;
use std::sync::mpsc::{Sender, Receiver, channel};

use notify::{Watcher, RecursiveMode, DebouncedEvent, RecommendedWatcher, watcher};
use npnc::{ProduceError, ConsumeError};
use npnc::bounded::spsc::{channel as npnc_channel, Producer, Consumer};

use atom::Atom;

/*
* 监听选项
*/
#[derive(Debug, Clone)]
pub enum FSMonitorOptions {
    File(Atom, u64),               //监听单个文件，路径和缓冲时间
    Files(Vec<(Atom, u64)>),       //监听多个文件，路径和缓冲时间的列表
    Dir(Atom, bool, u64),          //监听单个目录，路径、是否递归遍历和缓冲时间
    Dirs(Vec<(Atom, bool, u64)>),  //监听多个目录，路径、是否递归遍历和缓冲时间的列表
}

/*
* 文件改变事件
*/
#[derive(Debug, Clone)]
pub enum FSChangeEvent {
    Create(PathBuf),
    Write(PathBuf),
    Remove(PathBuf),
    Rename(PathBuf, PathBuf),
}

/*
* 监听者
*/
#[derive(Clone)]
pub struct FSListener(pub Arc<Fn(FSChangeEvent)>);

unsafe impl Send for FSListener {}

/*
* 监听器管理事件
*/
#[derive(Debug, Clone)]
enum FSMonitorEvent {
    Stop,           //关闭监听器
    Pause(usize),   //暂停监听器
}

/*
* 文件系统监听器
*/
pub struct FSMonitor {
    is_running: bool,                                   //是否正在运行
    options: FSMonitorOptions,                          //初始化选项
    watchers: HashMap<PathBuf, RecommendedWatcher>,     //监听器表
    listener: FSListener,                               //监听者
    watcher_sender: Option<Sender<DebouncedEvent>>,     //监听器消息发送器
    manager_sender: Option<Producer<FSMonitorEvent>>,   //管理消息发送器
}

impl Drop for FSMonitor {
    fn drop(&mut self) {
        //关闭监听器
        if let Err(e) = self.stop() {
            println!("!!!> Drop FSMonitor Error, e: {:?}", e);
        }

        //移除所有路径的监听
        for (path, mut watcher) in self.watchers.drain() {
            if let Err(e) = watcher.unwatch(path) {
                println!("!!!> Drop FSMonitor Error, e: {:?}", e);
            }
        }
    }
}

impl FSMonitor {
    //构建一个文件系统监听器
    pub fn new(options: FSMonitorOptions, listener: FSListener) -> Self {
        FSMonitor {
            is_running: false,
            options: options,
            watchers: HashMap::new(),
            listener: listener,
            watcher_sender: None,
            manager_sender: None,
        }
    }

    //检查是否监听了指定路径
    pub fn exists(&self, path: Atom) -> bool {
        let p = PathBuf::from(path.as_str());
        self.watchers.contains_key(&p)
    }

    //增加指定路径的监听
    pub fn add_monitor(&mut self, options: FSMonitorOptions) -> Result<(), String> {
        match self.watcher_sender.as_ref() {
            None => Err(format!("add fs monitor failed, invalid sender")),
            Some(sender) => add_monitor(&mut self.watchers, sender, &options),
        }
    }

    //移除指定路径的监听
    pub fn remove_monitor(&mut self, path: Atom) -> Result<(), String> {
        remove_monitor(&mut self.watchers, &PathBuf::from(path.to_string()))
    }

    //运行指定监听器
    pub fn run(&mut self) -> Result<(), String> {
        if self.is_running {
            return Err(format!("fs monitor run failed, already running"));
        }

        let (sender, receiver) = channel();
        let (p, c) = npnc_channel(1);
        match add_monitor(&mut self.watchers, &sender, &self.options) {
            Err(e) => return Err(e),
            Ok(_) => {
                self.watcher_sender = Some(sender);
                self.manager_sender = Some(p);
                let listener = self.listener.clone();
                thread::spawn(move || {
                    wait_recv(&receiver, &c, &listener);
                });
                self.is_running = true;
                Ok(())
            },
        }
    }

    //暂停监听器
    pub fn pause(&self, time: usize) -> Result<(), String> {
        if !self.is_running {
            return Err(format!("pause fs monitor failed, not running"));
        }

        match self.manager_sender.as_ref() {
            None => Err(format!("pause fs monitor failed, invalid sender")),
            Some(sender) => {
                match sender.produce(FSMonitorEvent::Pause(time)) {
                    Err(e) => {
                        match e {
                            ProduceError::Full(event) => Err(format!("pause fs monitor failed, event full, event: {:?}", event)),
                            ProduceError::Disconnected(event) => Err(format!("pause fs monitor failed, monitor closed, event: {:?}", event)),
                        }
                    },
                    Ok(_) => Ok(()),
                }
            },
        }
    }

    //关闭监听器
    pub fn stop(&self) -> Result<(), String> {
        if !self.is_running {
            return Err(format!("stop fs monitor failed, not running"));
        }

        match self.manager_sender.as_ref() {
            None => Err(format!("stop fs monitor failed, invalid sender")),
            Some(sender) => {
                match sender.produce(FSMonitorEvent::Stop) {
                    Err(e) => {
                        match e {
                            ProduceError::Full(event) => Err(format!("stop fs monitor failed, event full, event: {:?}", event)),
                            ProduceError::Disconnected(event) => Err(format!("stop fs monitor failed, monitor closed, event: {:?}", event)),
                        }
                    },
                    Ok(_) => Ok(()),
                }
            },
        }
    }
}

//判断监听路径是否是文件
fn is_file(path: &PathBuf) -> bool {
    path.is_file() && path.exists()
}

//判断监听路径是否是目录
fn is_dir(path: &PathBuf) -> bool {
    path.is_dir() && path.exists()
}

//增加监听器
fn add_monitor(watchers: &mut HashMap<PathBuf, RecommendedWatcher>, sender: &Sender<DebouncedEvent>, options: &FSMonitorOptions) 
    -> Result<(), String> {
        let mut path: PathBuf;
        match options {
            FSMonitorOptions::File(file, time) => {
                path = PathBuf::from(file.as_str());
                if is_file(&path) {
                    if let Err(e) = monitor_path(watchers, &sender, path, false, time.clone()) {
                        return Err(e);
                    }
                } else {
                    return Err(format!("monitor file error, invalid file, path: {:?}", &path));
                }
            },
            FSMonitorOptions::Files(files) => {
                for (file, time) in files {
                    path = PathBuf::from(file.as_str());
                    if is_file(&path) {
                        if let Err(e) = monitor_path(watchers, &sender, path, false, time.clone()) {
                            return Err(e);
                        }
                    } else {
                        return Err(format!("monitor file error, invalid file, path: {:?}", &path));
                    }
                }
            },
            FSMonitorOptions::Dir(dir, is_rec, time) => {
                path = PathBuf::from(dir.as_str());
                if is_dir(&path) {
                    if let Err(e) = monitor_path(watchers, &sender, path, is_rec.clone(), time.clone()) {
                        return Err(e);
                    }
                } else {
                    return Err(format!("monitor dir error, invalid dir, path: {:?}", &path));
                }
            },
            FSMonitorOptions::Dirs(dirs) => {
                for (dir, is_rec, time) in dirs {
                    path = PathBuf::from(dir.as_str());
                    if is_dir(&path) {
                        if let Err(e) = monitor_path(watchers, &sender, path, is_rec.clone(), time.clone()) {
                            return Err(e);
                        }
                    } else {
                        return Err(format!("monitor dir error, invalid dir, path: {:?}", &path));
                    }
                }
            }
        }
        Ok(())
}

//监听指定路径
fn monitor_path(watchers: &mut HashMap<PathBuf, RecommendedWatcher>, sender: &Sender<DebouncedEvent>, path: PathBuf, is_rec: bool, time: u64) 
    -> Result<(), String> {
        if watchers.contains_key(&path) {
            //指定路径已监听
            return Err(format!("add fs monitor failed, path exists"));
        }

        match watcher(sender.clone(), Duration::from_millis(time)) {
            Err(e) => Err(format!("add fs monitor failed, path: {:?}, e: {:?}", path, e)),
            Ok(mut watcher) => {
                let mode = if is_rec {
                    RecursiveMode::Recursive
                } else {
                    RecursiveMode::NonRecursive
                };

                match watcher.watch(&path, mode) {
                    Err(e) => Err(format!("add fs monitor failed, path: {:?}, e: {:?}", path, e)),
                    Ok(_) => {
                        watchers.insert(path, watcher);
                        Ok(())
                    }
                }
            },
        }
}

//移除监听器
fn remove_monitor(watchers: &mut HashMap<PathBuf, RecommendedWatcher>, path: &PathBuf) -> Result<(), String> {
    match watchers.remove_entry(path) {
        None => Ok(()), //指定路径的监听不存在，则忽略
        Some((p, mut wathcer)) => {
            //指定路径的监听存在，则关闭监听
            match wathcer.unwatch(p) {
                Err(e) => Err(format!("remove fs monitor failed, path: {:?}, e: {:?}", path, e)),
                Ok(_) => Ok(()),
            }
        }
    }
}

//等待接收事件，并通知监听者
fn wait_recv(receiver: &Receiver<DebouncedEvent>, consumer: &Consumer<FSMonitorEvent>, listener: &FSListener) {
    loop {
        //处理管理事件
        match consumer.consume() {
            Err(e) => {
                match e {
                    ConsumeError::Disconnected => {
                        //所有者已关闭，则立即退出监听线程
                        println!("!!!> Close Fs Monitor, owner closed");
                        break;
                    },
                    ConsumeError::Empty => (), //没有管理事件，则忽略
                }
            },
            Ok(event) => {
                match event {
                    FSMonitorEvent::Stop => {
                        //所有者请求关闭监听线程
                        println!("!!!> Close Fs Monitor, owner request close");
                        break;
                    },
                    FSMonitorEvent::Pause(time) => {
                        //所有者请求暂停监听线程指定时长
                        thread::sleep(Duration::from_millis(time as u64));
                    }
                }
            }
        }

        //等待处理文件事件
        match receiver.recv() {
            Ok(DebouncedEvent::Write(path)) => {
                (listener.0)(FSChangeEvent::Write(path));
            },
            Ok(DebouncedEvent::Remove(path)) => {
                (listener.0)(FSChangeEvent::Remove(path));
            },
            Ok(DebouncedEvent::Create(path)) => {
                (listener.0)(FSChangeEvent::Create(path));
            },
            Ok(DebouncedEvent::Rename(src, dst)) => {
                (listener.0)(FSChangeEvent::Rename(src, dst));
            },
            Err(e) => {
                //对端已关闭，则立即退出监听线程
                println!("!!!> Close Fs Monitor, peer closed, e: {:?}", e);
                break;
            },
            _ => (),
        }
    }
}

