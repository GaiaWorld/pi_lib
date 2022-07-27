use std::fs;
use std::env;
use std::fs::File;
use std::sync::Arc;
use std::ops::BitOr;
use std::str::pattern::Pattern;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::io::{Error, Result, ErrorKind};
#[cfg(target_os = "linux")]
use std::os::unix::io::{FromRawFd, IntoRawFd};
#[cfg(target_os = "windows")]
use std::os::windows::io::{FromRawHandle, IntoRawHandle};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

use toml;
use serde_derive::{Deserialize, Serialize};
use dashmap::DashMap;
use log::{info, error};

use pi_async::lock::mutex_lock::Mutex;
use pi_hash::XHashMap;
use syn::Pat::Or;
use toml::value::Index;
use crate::DEAFULT_MACRO_EXPAND_FILE_SUFFIX;

/*
* 源码目录名
*/
pub const SRC_DIR_NAME: &str = "src";

/*
* 库入口文件名
*/
pub const LIB_FILE_NAME: &str = "lib.rs";

/*
* 模块入口文件名
*/
pub const MOD_FILE_NAME: &str = "mod.rs";

/*
* 构建配置文件名
*/
pub const BUILD_FILE_NAME: &str = "Cargo.toml";

/*
* 本地对象代理文件根目录名
*/
pub const NATIVE_OBJECT_PROXY_FILE_DIR_NAME: &str = "native";

/*
* Rust源码文件扩展名
*/
pub const RUST_SOURCE_FILE_EXTENSION: &str = "rs";

/*
* 检查指定路径是否是库的根路径，如果是则返回库信息和源码路径
*/
pub fn check_crate(path: &Path) -> Result<(CrateInfo, PathBuf)> {
    match fs::read_dir(path) {
        Err(e) => Err(e),
        Ok(mut dirs) => {
            let mut crate_info = None;
            let mut src_path = None;

            while let Some(entry) = dirs.next() {
                match entry {
                    Err(e) => return Err(e),
                    Ok(e) => {
                        if let Some(filename) = e.path().file_name() {
                            match filename.to_str() {
                                Some(BUILD_FILE_NAME) => {
                                    //分析库信息
                                    match fs::read_to_string(e.path()) {
                                        Err(e) => return Err(e),
                                        Ok(str) => {
                                            crate_info = Some(str.into());
                                        },
                                    }
                                },
                                Some(SRC_DIR_NAME) => {
                                    //记录源码路径
                                    src_path = Some(e.path());
                                },
                                _ => (),
                            }
                        }
                    },
                }
            }

            if let (Some(crate_info), Some(src_path)) = (crate_info, src_path) {
                Ok((crate_info, src_path))
            } else {
                Err(Error::new(ErrorKind::Other, format!("Check crate failed, path: {:?}, reason: invalid crate", path)))
            }
        },
    }
}

/*
* 导出的库
*/
#[derive(Debug)]
pub struct Crate {
    path:   PathBuf,            //库的本地绝对路径
    info:   CrateInfo,          //库信息
    source: Vec<ParseContext>,  //源码信息
}

unsafe impl Send for Crate {}

impl Crate {
    //构建导出的库
    pub fn new<P: AsRef<Path>>(path: P,
                               info: CrateInfo,
                               source: Vec<ParseContext>) -> Self {
        let path = path.as_ref().to_path_buf();
        Crate {
            path,
            info,
            source,
        }
    }

    //获取导出库的本地绝对路径
    pub fn get_path(&self) -> &Path {
        self.path.as_path()
    }

    //获取导出库的库信息
    pub fn get_info(&self) -> &CrateInfo {
        &self.info
    }

    //获取导出库的源码信息
    pub fn get_source(&self) -> &[ParseContext] {
        self.source.as_slice()
    }
}

/*
* 库信息
*/
#[derive(Debug, Deserialize, Serialize)]
pub struct CrateInfo {
    package:        PackageInfo,    //包信息
    dependencies:   toml::Value,    //依赖信息
}

unsafe impl Send for CrateInfo {}

impl From<&str> for CrateInfo {
    fn from(src: &str) -> Self {
        match toml::from_str(src) {
            Err(e) => panic!("Parse from Cargo.toml to info failed, reason: {:?}", e),
            Ok(info) => info,
        }
    }
}

impl From<String> for CrateInfo {
    fn from(src: String) -> Self {
        match toml::from_str(src.as_str()) {
            Err(e) => panic!("Parse from Cargo.toml to info failed, reason: {:?}", e),
            Ok(info) => info,
        }
    }
}

impl From<&CrateInfo> for String {
    fn from(src: &CrateInfo) -> Self {
        match toml::to_string(src) {
            Err(e) => panic!("Parse from info to Cargo.toml failed, reason: {:?}", e),
            Ok(string) => string,
        }
    }
}

impl From<CrateInfo> for String {
    fn from(src: CrateInfo) -> Self {
        match toml::to_string(&src) {
            Err(e) => panic!("Parse from info to Cargo.toml failed, reason: {:?}", e),
            Ok(string) => string,
        }
    }
}

impl CrateInfo {
    //构建库信息
    pub fn new(name: &str,
               version: &str,
               authors: Vec<&str>,
               edition: &str) -> Self {
        let package = PackageInfo {
            name: name.into(),
            version: version.into(),
            authors: authors.into(),
            edition: edition.into(),
        };
        let dependencies = toml::Value::Table(toml::value::Table::default());

        CrateInfo {
            package,
            dependencies,
        }
    }

    //获取包信息的只读引用
    pub fn get_package(&self) -> &PackageInfo {
        &self.package
    }

    //获取包信息的可写引用
    pub fn get_package_mut(&mut self) -> &mut PackageInfo {
        &mut self.package
    }

    //获取依赖信息
    pub fn get_depends<'a>(&'a self) -> Option<&'a toml::value::Table> {
        if let toml::Value::Table(table) = &self.dependencies {
            return Some(table);
        }

        None
    }

    //追加指定的依赖
    pub fn append_depend(&mut self, name: &str, value: toml::Value) {
        if let toml::Value::Table(table) = &mut self.dependencies {
            table.insert(name.to_string(), value);
        }
    }

    //移除指定的依赖
    pub fn remove_depend(&mut self, name: &str) -> Option<toml::Value> {
        if let toml::Value::Table(table) = &mut self.dependencies {
            return table.remove(name);
        }

        None
    }
}

/*
* 包信息
*/
#[derive(Debug, Deserialize, Serialize)]
pub struct PackageInfo {
    name:       toml::Value,    //库名
    version:    toml::Value,    //版本号
    authors:    toml::Value,    //作者
    edition:    toml::Value,    //版本
}

unsafe impl Send for PackageInfo {}

impl PackageInfo {
    //获取库名
    pub fn get_name(&self) -> String {
        if let toml::Value::String(name) = &self.name {
            return name.clone();
        }

        unimplemented!(); //不应该执行此分支
    }

    //设置库名
    pub fn set_name(&mut self, name: &str) {
        self.name = name.into();
    }

    //获取版本号
    pub fn get_version(&self) -> String{
        if let toml::Value::String(version) = &self.version {
            return version.clone();
        }

        unimplemented!(); //不应该执行此分支
    }

    //设置版本号
    pub fn set_version(&mut self, version: &str) {
        self.version = version.into();
    }

    //获取作者
    pub fn get_authors(&self) -> Vec<String> {
        let mut vec = Vec::new();
        if let toml::Value::Array(array) = &self.authors {
            for author in array {
                if let toml::Value::String(author) = author {
                    vec.push(author.clone());
                }
            }
        }

        vec
    }

    //设置作者
    pub fn set_authors(&mut self, authors: Vec<&str>) {
        self.authors = authors.into();
    }

    //获取版本
    pub fn get_edition(&self) -> String {
        if let toml::Value::String(edition) = &self.edition {
            return edition.clone();
        }

        unimplemented!(); //不应该执行此分支
    }

    //设置版本
    pub fn set_edition(&mut self, edition: &str) {
        self.edition = edition.into();
    }
}

/*
* 源码宏展开文件的路径
*/
#[derive(Clone)]
pub struct MacroExpandPathBuf(Arc<InnerMacroExpandPathBuf>);

unsafe impl Send for MacroExpandPathBuf {}
unsafe impl Sync for MacroExpandPathBuf {}

impl AsRef<Path> for MacroExpandPathBuf {
    fn as_ref(&self) -> &Path {
        self.0.path.as_path()
    }
}

impl MacroExpandPathBuf {
    //构建一个源码宏展开文件的路径
    pub fn new(path: PathBuf, is_expanded: bool) -> MacroExpandPathBuf {
        let inner = InnerMacroExpandPathBuf {
            path,
            is_expanded,
        };

        MacroExpandPathBuf(Arc::new(inner))
    }
}

struct InnerMacroExpandPathBuf{
    path:           PathBuf,    //宏展开文件的路径
    is_expanded:    bool,       //是否已经宏展开
}

impl Drop for InnerMacroExpandPathBuf {
    fn drop(&mut self) {
        if self.is_expanded {
            //移除指定的临时宏展开文件
            if let Err(e) = fs::remove_file(&self.path) {
                error!("Drop macro expand file failed, path: {:?}, reason: {:?}", self.path, e);
            }
        }
    }
}

/*
* 源码宏展开器
*/
#[derive(Clone)]
pub struct MacroExpander(Arc<InnerMacroExpander>);

unsafe impl Send for MacroExpander {}
unsafe impl Sync for MacroExpander {}

impl MacroExpander {
    //构建一个源码宏展开器
    pub fn new<P, S>(root_dir: P,
                     src_path: P,
                     suffix: S,
                     requires: Vec<String>) -> Self
        where P: AsRef<Path>, S: ToString {
        let inner = InnerMacroExpander {
            root: root_dir.as_ref().to_path_buf(),
            src: src_path.as_ref().to_path_buf(),
            suffix: suffix.to_string(),
            requires,
        };

        MacroExpander(Arc::new(inner))
    }

    //展开指定路径的源码文件，成功返回临时生成的宏展开后的源码文件的句柄
    #[cfg(target_os = "windows")]
    pub fn expand<P: AsRef<Path>>(&self, origin: P) -> Result<Option<MacroExpandPathBuf>> {
        if let Some(filename) = origin.as_ref().file_name() {
            match filename.to_str() {
                Some(LIB_FILE_NAME) => {
                    //忽略对库入口文件的宏展开，并立即返回
                    return Ok(None);
                },
                Some(MOD_FILE_NAME) => {
                    //忽略对模块入口文件的宏展开，并立即返回
                    return Ok(None);
                },
                Some(filename_str) => {
                    //检查当前文件名是否需要忽略宏展开
                    for require in &self.0.requires {
                        if filename_str != require.as_str() {
                            //忽略宏展开，并立即返回
                            return Ok(None);
                        }
                    }
                },
                _ => (),
            }
        }

        //生成指定源码文件加指定后缀的宏展开文件
        let parent_path = origin
            .as_ref()
            .parent()
            .expect("Expand source failed");
        let file_name_str = origin
            .as_ref()
            .file_prefix()
            .expect("Expand source failed")
            .to_str()
            .expect("Expand source failed")
            .to_string()
            + self.0.suffix.as_str();
        let mut file_name = PathBuf::from(file_name_str);
        file_name.set_extension(RUST_SOURCE_FILE_EXTENSION);
        let expanded_file_path = parent_path.join(file_name);
        let file = File::options()
            .write(true)
            .truncate(true)
            .create(true)
            .open(&expanded_file_path)?;
        let expanded_file_out = unsafe { Stdio::from_raw_handle(file.into_raw_handle()) };

        //执行指定源码文件的宏展开操作，并将宏展开后的源码写入宏展开文件
        let path = origin
            .as_ref()
            .strip_prefix(&self.0.src)
            .expect("Expand source failed");
        if let Some(path_str) = path.to_str() {
            //有效的源文件路径名
            let vec: Vec<&str> = path_str.split(".rs").collect();
            let arg = vec[0]
                .replace("/", "::")
                .replace("\\", "::");

            let output = Command::new("cargo")
                .current_dir(&self.0.root)
                .arg("expand")
                .arg(&arg)
                .stdout(expanded_file_out)
                .stderr(Stdio::inherit())
                .output()?;
            if !output.status.success() {
                //执行宏展开指令失败，则打印
                info!("Expand source failed, from: {:?}, to: {:?}, stdout: {:?}",
                    arg,
                    expanded_file_path,
                    String::from_utf8(output.stdout).expect("Expand source failed"));
                error!("Expand source failed, from: {:?}, to: {:?}, stderr: {:?}",
                    arg,
                    expanded_file_path,
                    String::from_utf8(output.stderr).expect("Expand source failed"));
                return Err(Error::new(ErrorKind::Other, format!("Expand source failed, from: {:?}, to: {:?}, reason: run cargo expand error", arg, expanded_file_path)));
            }

            //执行宏展开指定成功，则注册宏展开文件，并返回宏展开文件的句柄
            info!("Expand source ok, from: {:?}, to: {:?}, stdout: {:?}",
                arg,
                expanded_file_path,
                String::from_utf8(output.stdout).expect("Expand source failed"));
            Ok(Some(MacroExpandPathBuf::new(expanded_file_path, true)))
        } else {
            //无效的源文件路径名
            Err(Error::new(ErrorKind::Other, format!("Expand source failed, from: {:?}, to: {:?}, reason: invalid origin", origin.as_ref(), expanded_file_path)))
        }
    }
    #[cfg(target_os = "linux")]
    pub fn expand<P: AsRef<Path>>(&self,
                                  root_dir: P,
                                  src_path: P,
                                  origin: P) -> Result<Option<MacroExpandPathBuf>> {
        if let Some(filename) = origin.as_ref().file_name() {
            match filename.to_str() {
                Some(LIB_FILE_NAME) => {
                    //忽略对库入口文件的宏展开，并立即返回
                    return Ok(None);
                },
                Some(MOD_FILE_NAME) => {
                    //忽略对模块入口文件的宏展开，并立即返回
                    return Ok(None);
                },
                Some(filename_str) => {
                    //检查当前文件名是否需要忽略宏展开
                    for require in &self.0.requires {
                        if filename_str != require.as_str() {
                            //忽略宏展开，并立即返回
                            return Ok(None);
                        }
                    }
                },
                _ => (),
            }
        }

        //生成指定源码文件加指定后缀的宏展开文件
        let parent_path = origin
            .as_ref()
            .parent()
            .expect("Expand source failed");
        let file_name_str = origin
            .as_ref()
            .file_prefix()
            .expect("Expand source failed")
            .to_str()
            .expect("Expand source failed")
            .to_string()
            + self.0.suffix.as_str();
        if let Some(_) = file_name_str.find(DEAFULT_MACRO_EXPAND_FILE_SUFFIX) {
            //如果文件是临时生成的宏展开后的源码文件，则立即返回
            return Ok(None);
        }
        let mut file_name = PathBuf::from(file_name_str);
        file_name.set_extension(RUST_SOURCE_FILE_EXTENSION);
        let expanded_file_path = parent_path.join(file_name);
        let file = File::options()
            .write(true)
            .truncate(true)
            .create(true)
            .open(expanded_file_path)?;
        let expanded_file_out = unsafe { Stdio::from_raw_fd(file.into_raw_fd()) };

        //执行指定源码文件的宏展开操作，并将宏展开后的源码写入宏展开文件
        let path = origin
            .as_ref()
            .strip_prefix(&self.0.src)
            .expect("Expand source failed");
        if let Some(path_str) = path.to_str() {
            //有效的源文件路径名
            let vec: Vec<&str> = path_str.split(".rs").collect();
            let arg = vec[0]
                .replace("/", "::")
                .replace("\\", "::");

            let output = Command::new("cargo")
                .current_dir(&self.0.root)
                .arg("expand")
                .arg(&arg)
                .stdout(expanded_file_out)
                .stderr(Stdio::inherit())
                .output()?;
            if !output.status.success() {
                //执行宏展开指令失败，则打印
                info!("Expand source failed, from: {:?}, to: {:?}, stdout: {:?}",
                    arg,
                    expanded_file_path,
                    String::from_utf8(output.stdout).expect("Expand source failed"));
                error!("Expand source failed, from: {:?}, to: {:?}, stderr: {:?}",
                    arg,
                    expanded_file_path,
                    String::from_utf8(output.stderr).expect("Expand source failed"));
                return Err(Error::new(ErrorKind::Other, format!("Expand source failed, from: {:?}, to: {:?}, reason: run cargo expand error", arg, expanded_file_path)));
            }

            //执行宏展开指定成功，则注册宏展开文件，并返回宏展开文件的句柄
            info!("Expand source ok, from: {:?}, to: {:?}, stdout: {:?}",
                arg,
                expanded_file_path,
                String::from_utf8(output.stdout).expect("Expand source failed"));
            Ok(Some(MacroExpandPathBuf::new(expanded_file_path, true)))
        } else {
            //无效的源文件路径名
            Err(Error::new(ErrorKind::Other, format!("Expand source failed, from: {:?}, to: {:?}, reason: invalid origin", origin.as_ref(), expanded_file_path)))
        }
    }
}

// 内部源码宏展开器
struct InnerMacroExpander {
    root:       PathBuf,                //库的根路径
    src:        PathBuf,                //库的源码根路径
    suffix:     String,                 //临时生成的宏展开后的源码文件名后缀
    requires:   Vec<String>,            //需要宏展开的文件名列表
}

/*
* 解析上下文
*/
#[derive(Debug)]
pub struct ParseContext {
    origin:     PathBuf,            //上下文的源
    is_export:  bool,               //正在解析的条目是否需要导出
    imports:    Vec<ImportItem>,    //导入条目数组
    exports:    Vec<ExportItem>,    //导出条目数组
}

unsafe impl Send for ParseContext {}

impl ParseContext {
    //构建解析上下文
    pub fn new(origin: &Path) -> Self {
        ParseContext {
            origin: origin.to_path_buf(),
            is_export: false,
            imports: Vec::new(),
            exports: Vec::new(),
        }
    }

    //获取上下文的源
    pub fn get_origin(&self) -> &Path {
        self.origin.as_path()
    }

    //判断当前正在解析的条目是否需要导出
    pub fn is_export(&self) -> bool {
        self.is_export
    }

    //设置当前正在解析的条目是否需要导出
    pub fn set_is_export(&mut self, b: bool) {
        self.is_export = b;
    }

    //从尾部弹出一个导入条目
    pub fn pop_import(&mut self) -> Option<ImportItem> {
        self.imports.pop()
    }

    //从尾部推入一个导入条目
    pub fn push_import(&mut self, item: ImportItem) {
        self.imports.push(item);
    }

    //获取尾部导入条目的只读引用
    pub fn get_last_import(&self) -> Option<&ImportItem> {
        self.imports.last()
    }

    //获取尾部导入条目的可写引用
    pub fn get_last_import_mut(&mut self) -> Option<&mut ImportItem> {
        self.imports.last_mut()
    }

    //获取导入条目数组的只读引用
    pub fn get_imports(&self) -> &[ImportItem] {
        self.imports.as_slice()
    }

    //从尾部弹出一个导出条目
    pub fn pop_export(&mut self) -> Option<ExportItem> {
        self.exports.pop()
    }

    //向尾部推入一个导出条目
    pub fn push_export(&mut self, item: ExportItem) {
        self.exports.push(item);
    }

    //获取尾部导出条目的只读引用
    pub fn get_last_export(&self) -> Option<&ExportItem> {
        self.exports.last()
    }

    //获取尾部导出条目的可写引用
    pub fn get_last_export_mut(&mut self) -> Option<&mut ExportItem> {
        self.exports.last_mut()
    }

    //获取导出条目数组的只读引用
    pub fn get_exports(&self) -> &[ExportItem] {
        self.exports.as_slice()
    }
}

/*
* 导入的条目
*/
#[derive(Debug)]
pub enum ImportItem {
    Std(LibPath),   //标准库
    Thrid(LibPath), //第三方库
}

unsafe impl Send for ImportItem {}

impl ImportItem {
    //是否是标准库导入
    pub fn is_std(&self) -> bool {
        match self {
            ImportItem::Std(_) => true,
            _ => false,
        }
    }

    //是否是第三方库导入
    pub fn is_thrid(&self) -> bool {
        match self {
            ImportItem::Thrid(_) => true,
            _ => false,
        }
    }

    //获取导入条目的库名
    pub fn get_crate_name(&self) -> String {
        match self {
            ImportItem::Std(_lib) => {
                "std".to_string()
            },
            ImportItem::Thrid(lib) => {
                lib.get_name().clone()
            },
        }
    }
}

/*
* 库路径
*/
#[derive(Debug)]
pub struct LibPath {
    name:   String,                     //路径名
    alias:  Option<String>,             //别名
    next:   Option<Box<LibPathNext>>,   //下一个路径
}

unsafe impl Send for LibPath {}

impl LibPath {
    //构建Rust库
    pub fn new(name: String) -> Self {
        LibPath {
            name,
            alias: None,
            next: None,
        }
    }

    //获取路径名
    pub fn get_name(&self) -> &String {
        &self.name
    }

    //获取别名
    pub fn get_alias(&self) -> Option<&String> {
        self.alias.as_ref()
    }

    //设置别名
    pub fn set_alias(&mut self, alias: String) {
        self.alias = Some(alias);
    }

    //获取下一个路径
    pub fn next(&self) -> Option<&LibPathNext> {
        if let Some(boxed) = &self.next {
            Some(&*boxed)
        } else {
            None
        }
    }

    //增加下一个路径
    pub fn join(&mut self, next: LibPathNext) {
        self.next = Some(Box::new(next));
    }
}

/*
* 下一个库路径
*/
#[derive(Debug)]
pub enum LibPathNext {
    Path(LibPath),
    Group(Vec<LibPath>),
}

unsafe impl Send for LibPathNext {}

/*
* 导出的条目
*/
#[derive(Debug)]
pub enum ExportItem {
    StructItem(Struct),     //导出的结构体
    EnumItem(Enum),         //导出的枚举
    FunctionItem(Function), //导出的函数
    ConstItem(Const),       //导出的常量
}

unsafe impl Send for ExportItem {}

impl ExportItem {
    //获取导出的条目名称
    pub fn get_name(&self) -> Option<String> {
        match self {
            ExportItem::StructItem(s) => s.get_name().cloned(),
            ExportItem::EnumItem(e) => e.get_name().cloned(),
            ExportItem::FunctionItem(f) => f.get_name().cloned(),
            ExportItem::ConstItem(c) => c.get_name().cloned(),
        }
    }

    //获取导出的条目类型名称
    pub fn get_type_name(&self) -> Option<String> {
        if let Some(item_name) = self.get_name() {
            //条目名称存在
            let mut item_type = Type::new(item_name);

            if let Some(item_generic) = match self {
                ExportItem::StructItem(s) => s.get_generic(),
                ExportItem::EnumItem(e) => e.get_generic(),
                ExportItem::FunctionItem(f) => f.get_generic(),
                ExportItem::ConstItem(_c) => None,
            } {
                //有类型参数
                for (type_arg_name, _) in item_generic.get_ref() {
                    item_type.append_type_argument(Type::new(type_arg_name.clone()));
                }
            }

            Some(item_type.to_string())
        } else {
            //条目名称不存在
            None
        }
    }

    //追加导出条目的文档
    pub fn append_doc(&mut self, doc: String) {
        match self {
            ExportItem::StructItem(s) => {
                if let Some(document) = &mut s.doc {
                    document.append(doc);
                } else {
                    //导出结构体没有文档，则创建
                    s.doc = Some(Document::new(doc));
                }
            },
            ExportItem::EnumItem(e) => {
                if let Some(document) = &mut e.doc {
                    document.append(doc);
                } else {
                    //导出枚举没有文档，则创建
                    e.doc = Some(Document::new(doc));
                }
            },
            ExportItem::FunctionItem(f) => {
                if let Some(document) = &mut f.doc {
                    document.append(doc);
                } else {
                    //导出函数没有文档，则创建
                    f.doc = Some(Document::new(doc));
                }
            },
            ExportItem::ConstItem(c) => {
                if let Some(document) = &mut c.doc {
                    document.append(doc);
                } else {
                    //导出常量没有文档，则创建
                    c.doc = Some(Document::new(doc));
                }
            }
        }
    }

    //追加导出条目泛型名称
    pub fn append_generic(&mut self, name: String) {
        match self {
            ExportItem::StructItem(s) => {
                if let Some(generic) = &mut s.generic {
                    generic.append_name(name);
                } else {
                    //导出结构体没有泛型，则创建
                    s.generic = Some(Generic::new(name));
                }
            },
            ExportItem::EnumItem(e) => {
                if let Some(generic) = &mut e.generic {
                    generic.append_name(name);
                } else {
                    //导出枚举没有泛型，则创建
                    e.generic = Some(Generic::new(name));
                }
            },
            ExportItem::FunctionItem(f) => {
                if let Some(generic) = &mut f.generic {
                    generic.append_name(name);
                } else {
                    //导出函数没有泛型，则创建
                    f.generic = Some(Generic::new(name));
                }
            },
            _ => {
                //忽略不支持的条目追加泛型名称
                ()
            },
        }
    }

    //追加导出条目泛型的具体类型名称
    pub fn append_generic_type(&mut self, r#type: String) {
        match self {
            ExportItem::StructItem(s) => {
                if let Some(generic) = &mut s.generic {
                    generic.append_type(r#type);
                }
            },
            ExportItem::EnumItem(e) => {
                if let Some(generic) = &mut e.generic {
                    generic.append_type(r#type);
                }
            },
            ExportItem::FunctionItem(f) => {
                if let Some(generic) = &mut f.generic {
                    generic.append_type(r#type);
                }
            },
            _ => {
                //忽略不支持的条目追加泛型的具体类型名称
                ()
            },
        }
    }

    //追加导出条目实现的Trait名称
    pub fn append_trait_impl(&mut self, name: String) {
        match self {
            ExportItem::StructItem(s) => {
                if let Some(trait_impl) = &mut s.trait_impls {
                    trait_impl.append_name(name);
                } else {
                    //导出结构体没有Trait实现，则创建
                    s.trait_impls = Some(TraitImpls::new(name));
                }
            },
            ExportItem::EnumItem(e) => {
                if let Some(trait_impl) = &mut e.trait_impls {
                    trait_impl.append_name(name);
                } else {
                    //导出枚举没有Trait实现，则创建
                    e.trait_impls = Some(TraitImpls::new(name));
                }
            },
            _ => {
                //忽略不支持的条目实现的Trait名称
                ()
            },
        }
    }

    //追加导出条目实现的Trait方法
    pub fn append_trait_method(&mut self, function: Function) {
        match self {
            ExportItem::StructItem(s) => {
                if let Some(trait_impl) = &mut s.trait_impls {
                    trait_impl.append_method(function);
                }
            },
            ExportItem::EnumItem(e) => {
                if let Some(trait_impl) = &mut e.trait_impls {
                    trait_impl.append_method(function);
                }
            },
            _ => {
                //忽略不支持的条目追加导出的Trait方法
                ()
            },
        }
    }

    //追加导出条目实现的方法
    pub fn append_method(&mut self, function: Function) {
        match self {
            ExportItem::StructItem(s) => {
                if let Some(impls) = &mut s.impls {
                    impls.append_method(function);
                } else {
                    //导出结构体没有实现，则创建
                    s.impls = Some(Impls::new(function));
                }
            },
            ExportItem::EnumItem(e) => {
                if let Some(impls) = &mut e.impls {
                    impls.append_method(function);
                } else {
                    //导出枚举没有实现，则创建
                    e.impls = Some(Impls::new(function));
                }
            },
            _ => {
                //忽略不支持的条目追加导出的方法
                ()
            },
        }
    }

    //追加导出条目的常量
    pub fn append_const(&mut self, c: Const) {
        match self {
            ExportItem::StructItem(s) => {
                if let Some(consts) = &mut s.consts {
                    consts.append_const(c);
                } else {
                    //导出结构体没有常量列表，则创建
                    s.consts = Some(ConstList::new(c));
                }
            },
            ExportItem::EnumItem(e) => {
                if let Some(consts) = &mut e.consts {
                    consts.append_const(c);
                } else {
                    //导出枚举没有常量列表，则创建
                    e.consts = Some(ConstList::new(c));
                }
            },
            _ => {
                //忽略不支持的条目追加常量的方法
                ()
            },
        }
    }
}

/*
* 结构体
*/
#[derive(Debug, Clone)]
pub struct Struct {
    name:           Option<String>,     //结构体的名称
    doc:            Option<Document>,   //结构体文档
    generic:        Option<Generic>,    //结构体泛型
    trait_impls:    Option<TraitImpls>, //结构体的Trait实现
    impls:          Option<Impls>,      //结构体的实现
    consts:         Option<ConstList>,  //结构体的常量列表
}

unsafe impl Send for Struct {}

impl Struct {
    //构建结构体
    pub fn new() -> Self {
        Struct {
            name: None,
            doc: None,
            generic: None,
            trait_impls: None,
            impls: None,
            consts: None,
        }
    }

    //获取结构体名称
    pub fn get_name(&self) -> Option<&String> {
        self.name.as_ref()
    }

    //设置结构体名称
    pub fn set_name(&mut self, name: String) {
        self.name = Some(name);
    }

    //获取结构体文档
    pub fn get_doc(&self) -> Option<&Document> {
        self.doc.as_ref()
    }

    //设置结构体文档
    pub fn set_doc(&mut self, doc: Document) {
        self.doc = Some(doc);
    }

    //获取结构体泛型
    pub fn get_generic(&self) -> Option<&Generic> {
        self.generic.as_ref()
    }

    //设置结构体泛型
    pub fn set_generic(&mut self, generic: Generic) {
        self.generic = Some(generic);
    }

    //获取结构体的Trait实现
    pub fn get_trait_impls(&self) -> Option<&TraitImpls> {
        self.trait_impls.as_ref()
    }

    //设置结构体的Trait实现
    pub fn set_trait_impls(&mut self, trait_impl: TraitImpls) {
        self.trait_impls = Some(trait_impl);
    }

    //获取结构体的实现
    pub fn get_impls(&self) -> Option<&Impls> {
        self.impls.as_ref()
    }

    //设置结构体的实现
    pub fn set_impls(&mut self, impls: Impls) {
        self.impls = Some(impls);
    }

    //获取结构体的常量列表
    pub fn get_consts(&self) -> Option<&ConstList> {
        self.consts.as_ref()
    }

    //设置结构体的实现
    pub fn set_consts(&mut self, consts: ConstList) {
        self.consts = Some(consts);
    }

    //获取有泛型参数的结构体的所有具体类型的组合
    pub fn get_specific_structs(&self) -> Option<Vec<Struct>> {
        if let Some(name) = self.get_name() {
            //有名称
            if let Some(generic) = self.get_generic() {
                //结构体有泛型参数
                let mut specific_types = Vec::new();
                let mut type_names = Vec::with_capacity(generic.get_ref().len());
                for (_, vec) in generic.get_ref() {
                    type_names.push(vec.clone());
                }
                combine_generic_args(name, &mut specific_types, None, &mut type_names, 0);
                // println!("{}", name);
                // for specific_type in &specific_types {
                //     println!("\t{}", specific_type.to_string());
                // }

                let mut specific_structs = Vec::new();
                for specific_type in specific_types {
                    //构建具体类型的结构体
                    let mut specific_struct = Struct::new();

                    //设置具体类型的结构体的文档
                    if let Some(struct_doc) = self.get_doc() {
                        specific_struct.set_doc(struct_doc.clone());
                    }

                    //设置具体类型的结构体的名称
                    specific_struct.set_name(specific_type.to_string());

                    //设置具体类型的结构体的泛型参数
                    let mut index = 0;
                    let mut specific_generic = Generic::empty();
                    for generic_name in generic.get_names() {
                        //简化泛型参数，为每个具体类型的结构体的泛型参数设置唯一的具体类型
                        specific_generic.append_name(generic_name);
                        specific_generic.append_type(specific_type.get_type_args().unwrap()[index].to_string());
                        index += 1;
                    }
                    specific_struct.set_generic(specific_generic);

                    //设置具体类型的结构体的Trait实现
                    if let Some(trait_impls) = self.get_trait_impls() {
                        let mut specific_trait_impls = TraitImpls::empty();

                        for (trait_name, trait_functions) in trait_impls.get_ref() {
                            specific_trait_impls.append_name(trait_name.clone());

                            for trait_fucntion in trait_functions {
                                if let Some(specific_functions) = trait_fucntion.get_specific_functions() {
                                    for specific_function in specific_functions {
                                        specific_trait_impls.append_method(specific_function);
                                    }
                                }
                            }
                        }

                        specific_struct.set_trait_impls(specific_trait_impls);
                    }

                    //设置具体类型的结构体的实现
                    if let Some(impls) = self.get_impls() {
                        let mut specific_impls = Impls::empty();

                        for function in impls.get_ref() {
                            if let Some(specific_functions) = function.get_specific_functions() {
                                for specific_function in specific_functions {
                                    specific_impls.append_method(specific_function);
                                }
                            }
                        }

                        specific_struct.set_impls(specific_impls);
                    }

                    //设置具体类型的结构体的常量
                    if let Some(consts) = self.get_consts() {
                        specific_struct.set_consts(consts.clone());
                    }

                    specific_structs.push(specific_struct);
                }

                Some(specific_structs)
            } else {
                //结构体没有泛型参数
                None
            }
        } else {
            //无名称
            None
        }
    }
}

/*
* 枚举
*/
#[derive(Debug, Clone)]
pub struct Enum {
    name:           Option<String>,     //枚举的名称
    doc:            Option<Document>,   //枚举文档
    generic:        Option<Generic>,    //枚举泛型
    trait_impls:    Option<TraitImpls>, //枚举的Trait实现
    impls:          Option<Impls>,      //枚举的实现
    consts:         Option<ConstList>,  //枚举的常量列表
}

unsafe impl Send for Enum {}

impl Enum {
    //构建枚举
    pub fn new() -> Self {
        Enum {
            name: None,
            doc: None,
            generic: None,
            trait_impls: None,
            impls: None,
            consts: None,
        }
    }

    //获取枚举名称
    pub fn get_name(&self) -> Option<&String> {
        self.name.as_ref()
    }

    //设置枚举名称
    pub fn set_name(&mut self, name: String) {
        self.name = Some(name);
    }

    //获取枚举文档
    pub fn get_doc(&self) -> Option<&Document> {
        self.doc.as_ref()
    }

    //设置枚举文档
    pub fn set_doc(&mut self, doc: Document) {
        self.doc = Some(doc);
    }

    //获取枚举泛型
    pub fn get_generic(&self) -> Option<&Generic> {
        self.generic.as_ref()
    }

    //设置枚举泛型
    pub fn set_generic(&mut self, generic: Generic) {
        self.generic = Some(generic);
    }

    //获取枚举的Trait实现
    pub fn get_trait_impls(&self) -> Option<&TraitImpls> {
        self.trait_impls.as_ref()
    }

    //设置枚举的Trait实现
    pub fn set_trait_impls(&mut self, trait_impl: TraitImpls) {
        self.trait_impls = Some(trait_impl);
    }

    //获取枚举的实现
    pub fn get_impls(&self) -> Option<&Impls> {
        self.impls.as_ref()
    }

    //设置枚举的实现
    pub fn set_impls(&mut self, impls: Impls) {
        self.impls = Some(impls);
    }

    //获取枚举的常量列表
    pub fn get_consts(&self) -> Option<&ConstList> {
        self.consts.as_ref()
    }

    //设置枚举的实现
    pub fn set_consts(&mut self, consts: ConstList) {
        self.consts = Some(consts);
    }

    //获取有泛型参数的枚举的所有具体类型的组合
    pub fn get_specific_enums(&self) -> Option<Vec<Enum>> {
        if let Some(name) = self.get_name() {
            //有名称
            if let Some(generic) = self.get_generic() {
                //枚举有泛型参数
                let mut specific_types = Vec::new();
                let mut type_names = Vec::with_capacity(generic.get_ref().len());
                for (_, vec) in generic.get_ref() {
                    type_names.push(vec.clone());
                }
                combine_generic_args(name, &mut specific_types, None, &mut type_names, 0);
                // println!("{}", name);
                // for specific_type in &specific_types {
                //     println!("\t{}", specific_type.to_string());
                // }

                let mut specific_enums = Vec::new();
                for specific_type in specific_types {
                    //构建具体类型的枚举
                    let mut specific_enum = Enum::new();

                    //设置具体类型的枚举的文档
                    if let Some(enum_doc) = self.get_doc() {
                        specific_enum.set_doc(enum_doc.clone());
                    }

                    //设置具体类型的枚举的名称
                    specific_enum.set_name(specific_type.to_string());

                    //设置具体类型的枚举的泛型参数
                    let mut index = 0;
                    let mut specific_generic = Generic::empty();
                    for generic_name in generic.get_names() {
                        //简化泛型参数，为每个具体类型的枚举的泛型参数设置唯一的具体类型
                        specific_generic.append_name(generic_name);
                        specific_generic.append_type(specific_type.get_type_args().unwrap()[index].to_string());
                        index += 1;
                    }
                    specific_enum.set_generic(specific_generic);

                    //设置具体类型的枚举的Trait实现
                    if let Some(trait_impls) = self.get_trait_impls() {
                        let mut specific_trait_impls = TraitImpls::empty();

                        for (trait_name, trait_functions) in trait_impls.get_ref() {
                            specific_trait_impls.append_name(trait_name.clone());

                            for trait_fucntion in trait_functions {
                                if let Some(specific_functions) = trait_fucntion.get_specific_functions() {
                                    for specific_function in specific_functions {
                                        specific_trait_impls.append_method(specific_function);
                                    }
                                }
                            }
                        }

                        specific_enum.set_trait_impls(specific_trait_impls);
                    }

                    //设置具体类型的枚举的实现
                    if let Some(impls) = self.get_impls() {
                        let mut specific_impls = Impls::empty();

                        for function in impls.get_ref() {
                            if let Some(specific_functions) = function.get_specific_functions() {
                                for specific_function in specific_functions {
                                    specific_impls.append_method(specific_function);
                                }
                            }
                        }

                        specific_enum.set_impls(specific_impls);
                    }

                    //设置具体类型的枚举的常量
                    if let Some(consts) = self.get_consts() {
                        specific_enum.set_consts(consts.clone());
                    }

                    specific_enums.push(specific_enum);
                }

                Some(specific_enums)
            } else {
                //枚举没有泛型参数
                None
            }
        } else {
            //无名称
            None
        }
    }
}

/*
* 函数
*/
#[derive(Debug, Clone)]
pub struct Function {
    is_async:   bool,               //是否是异步函数
    name:       Option<String>,     //函数的名称
    doc:        Option<Document>,   //函数的文档
    generic:    Option<Generic>,    //函数的泛型
    input:      Option<FunArgs>,    //函数的入参
    output:     Option<Type>,       //函数的出参
}

unsafe impl Send for Function {}

impl Function {
    //构建函数
    pub fn new() -> Self {
        Function {
            is_async: false,
            name: None,
            doc: None,
            generic: None,
            input: None,
            output: None,
        }
    }

    //是否是异步方法
    pub fn is_async(&self) -> bool {
        self.is_async
    }

    //设置方法为异步方法
    pub fn set_async(&mut self) {
        self.is_async = true;
    }

    //是否是静态方法
    pub fn is_static(&self) -> bool {
        if let Some(args) = &self.input {
            match args.get_ref()[0].0.as_str() {
                "self" | "&self" | "&mut self" => false,
                _ => true,
            }
        } else {
            true
        }
    }

    //获取函数名称
    pub fn get_name(&self) -> Option<&String> {
        self.name.as_ref()
    }

    //设置函数名称
    pub fn set_name(&mut self, name: String) {
        self.name = Some(name);
    }

    //获取函数文档
    pub fn get_doc(&self) -> Option<&Document> {
        self.doc.as_ref()
    }

    //设置函数文档
    pub fn set_doc(&mut self, doc: Document) {
        self.doc = Some(doc);
    }

    //获取函数泛型
    pub fn get_generic(&self) -> Option<&Generic> {
        self.generic.as_ref()
    }

    //设置函数泛型
    pub fn set_generic(&mut self, generic: Generic) {
        self.generic = Some(generic);
    }

    //获取函数入参
    pub fn get_input(&self) -> Option<&FunArgs> {
        self.input.as_ref()
    }

    //设置函数入参
    pub fn set_input(&mut self, input: FunArgs) {
        self.input = Some(input);
    }

    //追加函数的入参
    pub fn append_input(&mut self, name: String, r#type: Type) {
        if let Some(input) = &mut self.input {
            input.append(name, r#type);
        } else {
            //导出函数没有函数入参，则创建
            self.input = Some(FunArgs::new(name, r#type));
        }
    }

    //获取函数出参
    pub fn get_output(&self) -> Option<&Type> {
        self.output.as_ref()
    }

    //设置函数出参
    pub fn set_output(&mut self, output: Type) {
        self.output = Some(output);
    }

    //获取有泛型参数的函数的所有具体类型的组合，可以指定目标对象的泛型参数
    pub fn get_specific_functions(&self) -> Option<Vec<Function>> {
        if let Some(name) = &self.name {
            //有名称
            if let Some(generic) = &self.generic {
                //函数有泛型参数
                let mut specific_types = Vec::new();
                let mut type_names = Vec::with_capacity(generic.get_ref().len());
                for (_, vec) in generic.get_ref() {
                    type_names.push(vec.clone());
                }
                combine_generic_args(name, &mut specific_types, None, &mut type_names, 0);
                // println!("{}", name);
                // for specific_type in &specific_types {
                //     println!("\t{}", specific_type.to_string());
                // }

                let mut specific_functions = Vec::new();
                for specific_type in specific_types {
                    let mut specific_function = Function::new();

                    //设置具体类型的函数是否异步
                    if self.is_async() {
                        specific_function.set_async();
                    }

                    //设置具体类型的函数的文档
                    if let Some(funciton_doc) = self.get_doc() {
                        specific_function.set_doc(funciton_doc.clone());
                    }

                    //设置具体类型的函数名
                    specific_function.set_name(self.get_name().unwrap().clone());

                    //设置具体类型的函数的泛型参数
                    let mut index = 0;
                    let mut specific_generic = Generic::empty();
                    for generic_name in generic.get_names() {
                        //简化泛型参数，为每个具体类型的结构体的泛型参数设置唯一的具体类型
                        specific_generic.append_name(generic_name);
                        specific_generic.append_type(specific_type.get_type_args().unwrap()[index].to_string());
                        index += 1;
                    }
                    specific_function.set_generic(specific_generic);

                    //设置具体类型的函数的入参
                    if let Some(input) = self.get_input() {
                        specific_function.set_input(input.clone());
                    }

                    //设置具体类型的函数的出参
                    if let Some(output) = self.get_output() {
                        specific_function.set_output(output.clone());
                    }

                    specific_functions.push(specific_function);
                }

                Some(specific_functions)
            } else {
                //函数没有泛型参数，则返回原函数
                Some(vec![self.clone()])
            }
        } else {
            //无名称
            None
        }
    }
}

/*
* 常量
*/
#[derive(Debug, Clone)]
pub struct Const {
    name:       Option<String>,     //常量的名称
    doc:        Option<Document>,   //常量的文档
    ty:         Option<Type>,       //常量的类型
    value:      Option<ConstValue>, //常量值
}

unsafe impl Send for Const {}

impl Const {
    //构建常量
    pub fn new() -> Self {
        Const {
            name: None,
            doc: None,
            ty: None,
            value: None,
        }
    }

    //获取常量名称
    pub fn get_name(&self) -> Option<&String> {
        self.name.as_ref()
    }

    //设置常量名称
    pub fn set_name(&mut self, name: String) {
        self.name = Some(name);
    }

    //获取常量文档
    pub fn get_doc(&self) -> Option<&Document> {
        self.doc.as_ref()
    }

    //设置常量文档
    pub fn set_doc(&mut self, doc: Document) {
        self.doc = Some(doc);
    }

    //获取常量的类型
    pub fn get_type(&self) -> Option<&Type> {
        self.ty.as_ref()
    }

    //设置常量的类型
    pub fn set_type(&mut self, r#type: Type) {
        self.ty = Some(r#type);
    }

    //获取常量值
    pub fn get_value(&self) -> Option<&ConstValue> {
        self.value.as_ref()
    }

    //设置常量值
    pub fn set_value(&mut self, value: ConstValue) {
        self.value = Some(value);
    }
}

//组合多个泛型参数的具体类型
fn combine_generic_args(init_item_name: &String,
                        specific_types: &mut Vec<Type>,
                        specific_type: Option<Type>,
                        generics: &Vec<Vec<TypeName>>,
                        generics_index: usize) {
    let specific_type = if let Some(specific_type) = specific_type {
        specific_type
    } else {
        Type::new(init_item_name.clone())
    };

    if generics_index < generics.len() {
        //泛型参数还未分析完，则继续
        let generic = &generics[generics_index]; //获取当前的泛型参数的具体类型列表

        for generic_name in generic {
            let mut specific_type_copy = specific_type.clone(); //复制具体类型
            specific_type_copy.append_type_argument(Type::new(generic_name.to_string())); //为具体类型的复制追加当前的泛型参数的一个具体类型
            combine_generic_args(init_item_name,
                                 specific_types,
                                 Some(specific_type_copy),
                                 generics,
                                 generics_index + 1);
        }
    } else {
        //已追加所有泛型参数，则记录
        specific_types.push(specific_type);
    }
}


/*
* 文档
*/
#[derive(Debug, Clone)]
pub struct Document(Vec<String>);

unsafe impl Send for Document {}

impl Document {
    //构建文档
    pub fn new(doc: String) -> Self {
        Document(vec![doc])
    }

    //获取文档的只读引用
    pub fn get_ref(&self) -> &[String] {
        self.0.as_slice()
    }

    //追加文档
    pub fn append(&mut self, doc: String) {
        self.0.push(doc);
    }
}

/*
* 泛型，记录了泛型的名称和泛型的具体类型名
*/
#[derive(Debug, Clone)]
pub struct Generic(Vec<(String, Vec<TypeName>)>);

unsafe impl Send for Generic {}

impl Generic {
    //创建空的泛型
    pub fn empty() -> Self {
        Generic(vec![])
    }

    //创建泛型
    pub fn new(name: String) -> Self {
        Generic(vec![(name, Vec::new())])
    }

    //获取所有泛型的组合
    pub fn combine(&self) -> usize {
        let mut combine = 1;

        for (_, types) in &self.0 {
            combine *= types.len();
        }

        combine
    }

    //获取所有的泛型名称
    pub fn get_names(&self) -> Vec<String> {
        self.0.iter().map(|(name, _)| {
            name.clone()
        }).collect()
    }

    //获取泛型的只读引用
    pub fn get_ref(&self) -> &[(String, Vec<TypeName>)] {
        self.0.as_slice()
    }

    //追加指定泛型的名称
    pub fn append_name(&mut self, name: String) {
        self.0.push((name, Vec::new()));
    }

    //追加指定泛型的具体类型名称
    pub fn append_type(&mut self, type_name: String) {
        if let Some((_, types)) = self.0.last_mut() {
            types.push(TypeName::new(type_name));
        }
    }
}

/*
* 类型，包括：类型名和类型的参数列表
*/
#[derive(Debug, Clone)]
pub struct Type(TypeName, Option<Vec<Type>>);

unsafe impl Send for Type {}

impl ToString for Type {
    fn to_string(&self) -> String {
        let mut string = self.0.get_name().clone();

        if let Some(type_args) = &self.1 {
            string += "<";
            let mut iterator = type_args.iter();
            string += iterator.next().unwrap().to_string().as_str();
            while let Some(type_arg) = iterator.next() {
                string += ", ";
                string += type_arg.to_string().as_str();
            }
            string += ">";
        }

        string
    }
}

impl Type {
    //构建类型
    pub fn new(type_name: String) -> Self {
        Type(TypeName::new(type_name), None)
    }

    //获取完整类型名
    pub fn get_type_name(&self) -> TypeName {
        if self.0.is_moveable() {
            TypeName::new(self.to_string())
        } else if self.0.is_only_read() {
            TypeName::new("&".to_string() +self.to_string().as_str())
        } else {
            TypeName::new("&mut ".to_string() + self.to_string().as_str())
        }
    }

    //获取部分类型名
    pub fn get_part_type_name(&self) -> &TypeName {
        &self.0
    }

    //获取类型的参数列表类型名
    pub fn get_type_arg_names(&self) -> Option<Vec<TypeName>> {
        if let Some(types) = self.1.as_ref() {
            let mut vec = Vec::with_capacity(types.len());
            for ty in types {
                vec.push(ty.get_type_name());
            }

            return Some(vec);
        }

        None
    }

    //获取类型的参数列表
    pub fn get_type_args(&self) -> Option<&Vec<Type>> {
        self.1.as_ref()
    }

    //追加类型参数
    pub fn append_type_argument(&mut self, arg: Type) {
        if let Some(type_args) = &mut self.1 {
            type_args.push(arg);
        } else {
            self.1 = Some(vec![arg]);
        }
    }
}

/*
* 类型名
*/
#[derive(Debug, Clone)]
pub enum TypeName {
    Moveable(String),
    OnlyRead(String),
    Writable(String),
}

unsafe impl Send for TypeName {}

impl ToString for TypeName {
    fn to_string(&self) -> String {
        match self {
            TypeName::Moveable(str) => str.clone(),
            TypeName::OnlyRead(str) => "&".to_string() + str,
            TypeName::Writable(str) => "&mut ".to_string() + str,
        }
    }
}

impl TypeName {
    //构建类型名
    pub fn new(name: String) -> Self {
        if name.starts_with("&mut") {
            TypeName::Writable(name.replace("&mut ", ""))
        } else if name.starts_with("&") {
            TypeName::OnlyRead(name.replace("&", ""))
        } else {
            TypeName::Moveable(name)
        }
    }

    //是否可移动
    pub fn is_moveable(&self) -> bool {
        if let TypeName::Moveable(_) = self {
            true
        } else {
            false
        }
    }

    //是否只读
    pub fn is_only_read(&self) -> bool {
        if let TypeName::OnlyRead(_) = self {
            true
        } else {
            false
        }
    }

    //是否可写
    pub fn is_writable(&self) -> bool {
        if let TypeName::Writable(_) = self {
            true
        } else {
            false
        }
    }

    //只获取类型的名称
    pub fn get_name(&self) -> &String {
        match self {
            TypeName::Moveable(str) => &str,
            TypeName::OnlyRead(str) => &str,
            TypeName::Writable(str) => &str,
        }
    }
}

/*
* 函数参数，包括：参数名，参数类型名称和参数类型的类型列表
*/
#[derive(Debug, Clone)]
pub struct FunArgs(Vec<(String, Type)>);

unsafe impl Send for FunArgs {}

impl FunArgs {
    //构建函数参数
    pub fn new(arg_name: String, arg_type: Type) -> Self {
        FunArgs(vec![(arg_name, arg_type)])
    }

    //获了函数参数数量
    pub fn len(&self) -> usize {
        self.0.len()
    }

    //获取函数参数的只读引用
    pub fn get_ref(&self) -> &[(String, Type)] {
        self.0.as_slice()
    }

    //追加指定的函数参数
    pub fn append(&mut self, arg_name: String, arg_type: Type) {
        self.0.push((arg_name, arg_type));
    }
}

/*
* Trait实现
*/
#[derive(Debug, Clone)]
pub struct TraitImpls(Vec<(String, Vec<Function>)>);

unsafe impl Send for TraitImpls {}

impl TraitImpls {
    //构建空的Trait实现
    pub fn empty() -> Self {
        TraitImpls(vec![])
    }

    //构建Trait实现
    pub fn new(name: String) -> Self {
        TraitImpls(vec![(name, Vec::new())])
    }

    //获取所有Trait名称
    pub fn get_names(&self) -> Vec<String> {
        self.0.iter().map(|(name, _)| {
            name.clone()
        }).collect()
    }

    //获取Trait实现的只读引用
    pub fn get_ref(&self) -> &[(String, Vec<Function>)] {
        self.0.as_slice()
    }

    //追加Trait名称
    pub fn append_name(&mut self, name: String) {
        self.0.push((name, Vec::new()));
    }

    //追加Trait方法
    pub fn append_method(&mut self, function: Function) {
        if let Some((_, methods)) = self.0.last_mut() {
            methods.push(function);
        }
    }
}

/*
* 实现
*/
#[derive(Debug, Clone)]
pub struct Impls(Vec<Function>);

unsafe impl Send for Impls {}

impl Impls {
    //构建空的实现
    pub fn empty() -> Self {
        Impls(vec![])
    }

    //构建Trait实现
    pub fn new(function: Function) -> Self {
        Impls(vec![function])
    }

    //获取实现的只读引用
    pub fn get_ref(&self) -> &[Function] {
        self.0.as_slice()
    }

    //追加方法
    pub fn append_method(&mut self, function: Function) {
        self.0.push(function);
    }
}

/*
* 常量列表
*/
#[derive(Debug, Clone)]
pub struct ConstList(Vec<Const>);

unsafe impl Send for ConstList {}

impl ConstList {
    //构建常量列表
    pub fn new(c: Const) -> Self {
        ConstList(vec![c])
    }

    //获取常量列表的只读引用
    pub fn get_ref(&self) -> &[Const] {
        self.0.as_slice()
    }

    //追加常量
    pub fn append_const(&mut self, c: Const) {
        self.0.push(c);
    }
}

/*
* 常量值
*/
#[derive(Debug, Clone)]
pub enum ConstValue {
    Boolean(bool),  //布尔值
    Int(i64),       //有符号整数
    Uint(i64),      //无符号整数
    Float(f64),     //浮点数
    Str(String),    //字符串
}

unsafe impl Send for ConstValue {}

impl ToString for ConstValue {
    fn to_string(&self) -> String {
        match self {
            ConstValue::Boolean(b) => b.to_string(),
            ConstValue::Int(num) => num.to_string(),
            ConstValue::Uint(num) => num.to_string(),
            ConstValue::Float(num) => num.to_string(),
            ConstValue::Str(str) => "\"".to_string() + str + "\"",
        }
    }
}

impl ConstValue {
    //获取常量值对应的ts类型名称
    pub fn get_ts_type_name(&self) -> String {
        match self {
            ConstValue::Boolean(_) => "boolean".to_string(),
            ConstValue::Int(_) => "number".to_string(),
            ConstValue::Uint(_) => "number".to_string(),
            ConstValue::Float(_) => "number".to_string(),
            ConstValue::Str(_) => "string".to_string(),
        }
    }
}

/*
* 属性词条过滤器
*/
#[derive(Debug, Clone)]
pub enum AttributeTokensFilter {
    _None    = 0,    //不过滤
    Punct   = 1,    //过滤标识符号
    Ident   = 2,    //过滤标识符
    Literal = 4,    //过滤字面量
    Group   = 8,    //过滤词条数组
}

impl BitOr for AttributeTokensFilter {
    type Output = u8;

    fn bitor(self, rhs: Self) -> Self::Output {
        if let AttributeTokensFilter::_None = self {
            //任何过滤与不过滤进行或运算，都等于不过滤
            return AttributeTokensFilter::_None as u8;
        } else if let AttributeTokensFilter::_None = rhs {
            //任何过滤与不过滤进行或运算，都等于不过滤
            return AttributeTokensFilter::_None as u8;
        } else {
            (self as u8) | (rhs as u8)
        }
    }
}

impl AttributeTokensFilter {
    //是否不过滤
    pub fn is_no(filter: u8) -> bool {
        filter == 0
    }

    //是否只过滤标识符号
    pub fn is_punct(filter: u8) -> bool {
        (filter & (AttributeTokensFilter::Punct as u8)) != 0
    }

    //是否只过滤标识符
    pub fn is_ident(filter: u8) -> bool {
        (filter & (AttributeTokensFilter::Ident as u8)) != 0
    }

    //是否只过滤字面量
    pub fn is_literal(filter: u8) -> bool {
        (filter & (AttributeTokensFilter::Literal as u8)) != 0
    }

    //是否只过滤词条数组
    pub fn is_group(filter: u8) -> bool {
        (filter & (AttributeTokensFilter::Group as u8)) != 0
    }
}

/*
* 分析具体类型的栈帧
*/
#[derive(Debug)]
pub enum WithParseSpecificTypeStackFrame {
    Punct(char),    //标点符号
    Type(Type),     //类型
}

/*
* 代理源码生成器
*/
#[derive(Clone)]
pub struct ProxySourceGenerater {
    static_method_index:        Arc<AtomicUsize>,                               //同步静态代理方法序号
    async_static_method_index:  Arc<AtomicUsize>,                               //异步静态代理方法序号
    method_index:               Arc<AtomicUsize>,                               //同步代理方法序号
    async_method_index:         Arc<AtomicUsize>,                               //异步代理方法序号
    export_mods:                Arc<Mutex<Vec<String>>>,                        //需要在lib中导出的模块名列表
    static_methods:             Arc<Mutex<Vec<String>>>,                        //需要注册的同步静态代理方法名列表
    async_static_methods:       Arc<Mutex<Vec<String>>>,                        //需要注册的异步静态代理方法名列表
    methods:                    Arc<Mutex<Vec<String>>>,                        //需要注册的同步代理方法名列表
    async_methods:              Arc<Mutex<Vec<String>>>,                        //需要注册的异步代理方法名列表
    static_methods_map:         Arc<Mutex<XHashMap<(String, String), usize>>>,  //同步静态代理方法反向映射表
    async_static_methods_map:   Arc<Mutex<XHashMap<(String, String), usize>>>,  //异步静态代理方法反向映射表
    methods_map:                Arc<Mutex<XHashMap<(String, String), usize>>>,  //同步代理方法反向映射表
    async_methods_map:          Arc<Mutex<XHashMap<(String, String), usize>>>,  //异步代理方法反向映射表
}

unsafe impl Send for ProxySourceGenerater {}
unsafe impl Sync for ProxySourceGenerater {}

/*
* 代理源码生成器同步方法
*/
impl ProxySourceGenerater {
    //构建代理源码生成器
    pub fn new() -> Self {
        ProxySourceGenerater {
            static_method_index: Arc::new(AtomicUsize::new(0)),
            async_static_method_index: Arc::new(AtomicUsize::new(0)),
            method_index: Arc::new(AtomicUsize::new(0)),
            async_method_index: Arc::new(AtomicUsize::new(0)),
            export_mods: Arc::new(Mutex::new(Vec::new())),
            static_methods: Arc::new(Mutex::new(Vec::new())),
            async_static_methods: Arc::new(Mutex::new(Vec::new())),
            methods: Arc::new(Mutex::new(Vec::new())),
            async_methods: Arc::new(Mutex::new(Vec::new())),
            static_methods_map: Arc::new(Mutex::new(XHashMap::default())),
            async_static_methods_map: Arc::new(Mutex::new(XHashMap::default())),
            methods_map: Arc::new(Mutex::new(XHashMap::default())),
            async_methods_map: Arc::new(Mutex::new(XHashMap::default())),
        }
    }
}

/*
* 代理源码生成器异步方法
*/
impl ProxySourceGenerater {
    //获取需要在lib中导出的模块名列表
    pub async fn take_export_mods(&self) -> Vec<String> {
        self.export_mods.lock().await.clone()
    }

    //追加需要在lib中导出的模块名
    pub async fn append_export_mod(&self, name: String) {
        self.export_mods.lock().await.push(name);
    }

    //获取需要注册的同步静态代理方法名列表
    pub async fn take_static_methods(&self) -> Vec<String> {
        self.static_methods.lock().await.clone()
    }

    //追加需要注册的同步静态代理方法名，返回分配的同步静态代理方法序号
    pub async fn append_static_method(&self,
                                      target_name: Option<&String>,
                                      origin_name: String,
                                      proxy_name: String) -> usize {
        let method_index;

        {
            let mut static_methods = self.static_methods.lock().await;
            method_index = self.static_method_index.fetch_add(1, Ordering::Relaxed);
            let method_name = proxy_name + method_index.to_string().as_str();
            static_methods.push(method_name);
        }

        if let Some(target_name) = target_name {
            self.static_methods_map.lock().await.insert((target_name.clone(), origin_name), method_index);
        } else {
            self.static_methods_map.lock().await.insert(("".to_string(), origin_name), method_index);
        }

        method_index
    }

    //获取需要注册的异步静态代理方法名列表
    pub async fn take_async_static_methods(&self) -> Vec<String> {
        self.async_static_methods.lock().await.clone()
    }

    //追加需要注册的异步静态代理方法名，返回分配的异步静态代理方法序号
    pub async fn append_async_static_method(&self,
                                            target_name: Option<&String>,
                                            origin_name: String,
                                            proxy_name: String) -> usize {
        let method_index;

        {
            let mut async_static_methods = self.async_static_methods.lock().await;
            method_index = self.async_static_method_index.fetch_add(1, Ordering::Relaxed);
            let method_name = proxy_name + method_index.to_string().as_str();
            async_static_methods.push(method_name);
        }

        if let Some(target_name) = target_name {
            self.async_static_methods_map.lock().await.insert((target_name.clone(), origin_name), method_index);
        } else {
            self.async_static_methods_map.lock().await.insert(("".to_string(), origin_name), method_index);
        }

        method_index
    }

    //获取需要注册的同步代理方法名列表
    pub async fn take_methods(&self) -> Vec<String> {
        self.methods.lock().await.clone()
    }

    //追加需要注册的同步代理方法名，返回分配的同步代理方法序号
    pub async fn append_method(&self,
                               target_name: Option<&String>,
                               origin_name: String,
                               proxy_name: String) -> usize {
        let method_index;

        {
            let mut methods = self.methods.lock().await;
            method_index = self.method_index.fetch_add(1, Ordering::Relaxed);
            let method_name = proxy_name + method_index.to_string().as_str();
            methods.push(method_name);
        }

        if let Some(target_name) = target_name {
            self.methods_map.lock().await.insert((target_name.clone(), origin_name), method_index);
        } else {
            self.methods_map.lock().await.insert(("".to_string(), origin_name), method_index);
        }

        method_index
    }

    //获取需要注册的异步代理方法名列表
    pub async fn take_async_methods(&self) -> Vec<String> {
        self.async_methods.lock().await.clone()
    }

    //追加需要注册的异步代理方法名，返回分配的异步代理方法序号
    pub async fn append_async_method(&self,
                                     target_name: Option<&String>,
                                     origin_name: String,
                                     proxy_name: String) -> usize {
        let method_index;

        {
            let mut async_methods = self.async_methods.lock().await;
            method_index = self.async_method_index.fetch_add(1, Ordering::Relaxed);
            let method_name = proxy_name + method_index.to_string().as_str();
            async_methods.push(method_name);
        }

        if let Some(target_name) = target_name {
            self.async_methods_map.lock().await.insert((target_name.clone(), origin_name), method_index);
        } else {
            self.async_methods_map.lock().await.insert(("".to_string(), origin_name), method_index);
        }

        method_index
    }

    //获取指定目标对象名称和具体函数名称的同步静态方法序号
    pub async fn get_static_method_index(&self,
                                         target_name: String,
                                         origin_name: String) -> Option<usize> {
        self.static_methods_map.lock().await.get(&(target_name, origin_name)).cloned()
    }

    //获取指定目标对象名称和具体函数名称的异步静态方法序号
    pub async fn get_async_static_method_index(&self,
                                               target_name: String,
                                               origin_name: String) -> Option<usize> {
        self.async_static_methods_map.lock().await.get(&(target_name, origin_name)).cloned()
    }

    //获取指定目标对象名称和具体函数名称的同步方法序号
    pub async fn get_method_index(&self,
                                  target_name: String,
                                  origin_name: String) -> Option<usize> {
        self.methods_map.lock().await.get(&(target_name, origin_name)).cloned()
    }

    //获取指定目标对象名称和具体函数名称的异步方法序号
    pub async fn get_async_method_index(&self,
                                        target_name: String,
                                        origin_name: String) -> Option<usize> {
        self.async_methods_map.lock().await.get(&(target_name, origin_name)).cloned()
    }
}

/*
* 获取指定路径的绝对路径
*/
pub fn abs_path(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        //已经是绝对路径，则忽略
        return Ok(path.to_path_buf());
    }

    let cwd = env::current_dir()?;
    match if cfg!(windows) {
        path.strip_prefix(r#".\"#)
    } else {
        path.strip_prefix("./")
    }{
        Err(_e) => {
            Ok(cwd.join(path))
        },
        Ok(path) => {
            Ok(cwd.join(path))
        },
    }
}

/*
* 生成指定层数的tab
*/
pub fn create_tab(mut level: isize) -> String {
    let mut tab = String::new();

    while level > 0 {
        tab += "\t";
        level -= 1;
    }

    tab
}

/*
* 生成临时变量名
*/
#[inline]
pub fn create_tmp_var_name(index: usize) -> String {
    "val_".to_string() + index.to_string().as_str()
}

//获取目标类型的类型名，不包括类型参数
pub fn get_target_type_name(target_name: &String) -> String {
    let mut vec: Vec<String> = target_name.split('<').map(|x| {
        x.to_string()
    }).collect();

    vec.remove(0)
}

//获取具体函数名，如果有泛型参数，则根据泛型参数的具体类型生成具体函数名
pub fn get_specific_ts_function_name(function: &Function) -> String {
    let mut function_name = function.get_name().unwrap().clone();
    if let Some(generic) = function.get_generic() {
        //有泛型参数
        for (_, specific_types) in generic.get_ref() {
            let specific_type = specific_types[0].to_string()
                .replace("[", "$")
                .replace("]", "$")
                .replace("<", "_")
                .replace(", ", "_")
                .replace(",", "_")
                .replace(">", "_");
            function_name = function_name + "_" + specific_type.as_str();
        }
    }

    function_name
}

//获取具体类名，如果有泛型参数，则根据泛型参数的具体类型生成具体类名
pub fn get_specific_ts_class_name(class_name: &String) -> String {
    class_name
        .replace("[", "$")
        .replace("]", "$")
        .replace("<", "_")
        .replace(", ", "_")
        .replace(",", "_")
        .replace(">", "_")
}

#[test]
fn test_macro_expander() {
    use env_logger;

    //启动日志系统
    env_logger::builder().format_timestamp_millis().init();

    let mut expander = MacroExpander::new(r#"E:\wsl_tmp\pi_ui_render"#,
                                          r#"E:\wsl_tmp\pi_ui_render\src"#,
                                          "__$expand$__",
                                          vec!["style.rs".to_string()]);
    match expander.expand(r#"E:\wsl_tmp\pi_ui_render\src\export\mod.rs"#) {
        Err(e) => panic!("{:?}", e),
        Ok(None) => println!("!!!!!!ignore mod file"),
        Ok(Some(path)) => {
            println!("!!!!!!to: {:?}", path.as_ref());
        }
    }

    match expander.expand(r#"E:\wsl_tmp\pi_ui_render\src\export\style.rs"#) {
        Err(e) => panic!("{:?}", e),
        Ok(None) => panic!("Invalid source file"),
        Ok(Some(path)) => {
            println!("!!!!!!to: {:?}", path.as_ref());
        }
    }
}