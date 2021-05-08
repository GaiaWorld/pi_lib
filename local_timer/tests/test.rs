use std::collections::VecDeque;

use dyn_uint::{ClassFactory, UintFactory};
use time::run_millis;
use local_timer::{frame_wheel::{FrameWheel, LevelEnum, next_tail}, item::TimeoutItem, local_timer::LocalTimer};
use std::mem::{replace};

#[test]
fn test(){
    let mut wheel = LocalTimer::<i32, 100, 60, 60, 24>::new(10, run_millis());
    let times = [0, 10, 1000, 1010, 3000, 3100, 50, 60000, 61000, 3600000, 3500000, 86400000, 86600000];

    let mut now = 0 as u64;

    //测试插入到轮中的元素位置是否正确
    for v in times.iter(){
        wheel.insert(v.clone(), v.clone() as u64);
    }

    //测试插入到堆中的元素位置是否正确
    let heap_elem = 90061010;
    wheel.insert(heap_elem, heap_elem as u64);
    
    let opt = pop_test(&mut wheel, now);
    if let Some(rr) = opt {
        assert_eq!(wheel.get_item_timeout(&rr.0), 0);
    }
    else { 
        panic!("pop error");
    }

    //滚动一次， 只有时间为10毫秒的元素被取出
    now += wheel.frame_time;
    let r = roll(&mut wheel, now);
    assert_eq!(r.len(), 1);
    assert_eq!(wheel.get_item_timeout(&r[0].0), 10);

    //滚动三次， 不能取出任何元素
    for _i in 1..4{
        now += wheel.frame_time;
        let r =roll(&mut wheel, now);
        assert_eq!(r.len(), 0);
    }

    //滚动1次， 只有时间为50毫秒的元素被取出（滚动第五次）
    now += wheel.frame_time;
    let r = roll(&mut wheel, now);
    assert_eq!(r.len(), 1);
    assert_eq!(wheel.get_item_timeout(&r[0].0), 50);

    //滚动94次， 不能取出任何元素（滚动到第99次）
    for _i in 1..95{
        now += wheel.frame_time;
        let r = roll(&mut wheel, now);
        assert_eq!(r.len(), 0);
    }

    //滚动1次， 只有时间为1000毫秒的元素被取出（滚动到第100次）
    now += wheel.frame_time;
    let r = roll(&mut wheel, now);
    assert_eq!(r.len(), 1);
    assert_eq!(wheel.get_item_timeout(&r[0].0), 1000);
    
    //滚动1次， 只有时间为1010毫秒的元素被取出（滚动到第101次）
    now += wheel.frame_time;
    let r = roll(&mut wheel, now);
    assert_eq!(r.len(), 1);
    assert_eq!(wheel.get_item_timeout(&r[0].0), 1010);

    //滚动199次， 不能取出任何元素（滚动到第299次）
    for _i in 1..199 {
        now += wheel.frame_time;
        let r = roll(&mut wheel, now);
        assert_eq!(r.len(), 0);
    }

    //滚动1次， 只有时间为3000毫秒的元素被取出（滚动到第300次）
    now += wheel.frame_time;
    let r = roll(&mut wheel, now);
    assert_eq!(r.len(), 1);
    assert_eq!(wheel.get_item_timeout(&r[0].0), 3000);

    //滚动10次， 只有时间为3100毫秒的元素被取出（滚动到第310次）
    for _i in 1..10 {
        now += wheel.frame_time;
        let r = roll(&mut wheel, now);
        assert_eq!(r.len(), 0);
    }
    now += wheel.frame_time;
    let r = roll(&mut wheel, now);
    assert_eq!(r.len(), 1);
    assert_eq!(wheel.get_item_timeout(&r[0].0), 3100);
    
    // 移除 60000
    let r = wheel.remove(8);
    assert_eq!(wheel.get_item_timeout(&r.unwrap()), 60000);
    
    //滚动6100 - 310次， 只有时间为 61000 毫秒的元素被取出（滚动到第 6100 次）
    let temp = 61000 / 10 - 310;
    for _i in 1..temp {
        now += wheel.frame_time;
        let r = roll(&mut wheel, now);
        if r.len() == 1 {
            println!("Temp {:?} - now {:?} - timeout {:?}", _i, now, wheel.get_item_timeout(&r[0].0));
        }
        assert_eq!(r.len(), 0);
    }
    now += wheel.frame_time;
    let r = roll(&mut wheel, now);
    assert_eq!(r.len(), 1);
    assert_eq!(wheel.get_item_timeout(&r[0].0), 61000);

    //滚动 350000 - 6100 次， 只有时间为 3500000 毫秒的元素被取出（滚动到第 350000 次）
    let temp = 3500000 / 10 - 6100;
    println!("Temp {:?}", temp);
    for _i in 1..temp {
        now += wheel.frame_time;
        let r = roll(&mut wheel, now);
        if r.len() == 1 {
            println!("Temp {:?} - now {:?}", _i + 6100, now);
        }
        assert_eq!(r.len(), 0);
    }
    now += wheel.frame_time;
    let r = roll(&mut wheel, now);
    assert_eq!(r.len(), 1);
    assert_eq!(wheel.get_item_timeout(&r[0].0), 3500000);

    let r = wheel.remove(12);
    assert_eq!(wheel.get_item_timeout(&r.unwrap()), 86400000);
    
    //滚动 360000 - 350000 次， 只有时间为 3600000 毫秒的元素被取出（滚动到第 360000 次）
    let temp = 3600000 / 10 - 350000;
    println!("Temp {:?}", temp);
    for _i in 1..temp {
        now += wheel.frame_time;
        let r = roll(&mut wheel, now);
        if r.len() == 1 {
            println!("Temp {:?} - now {:?}", _i + 350000, now);
        }
        assert_eq!(r.len(), 0);
    }
    now += wheel.frame_time;
    let r = roll(&mut wheel, now);
    assert_eq!(r.len(), 1);
    assert_eq!(wheel.get_item_timeout(&r[0].0), 3600000);

    //滚动 8660000 - 360000 次， 只有时间为 86600000 毫秒的元素被取出（滚动到第 8660000 次）
    let temp = 86600000 / 10 - 360000;
    println!("Temp {:?}", temp);
    for _i in 1..temp {
        now += wheel.frame_time;
        let r = roll(&mut wheel, now);
        if r.len() == 1 {
            println!("Temp {:?} - now {:?} - point {:?}", _i + 360000, now, wheel.get_item_timeout(&r[0].0));
        }
        assert_eq!(r.len(), 0);
    }
    now += wheel.frame_time;
    let r = roll(&mut wheel, now);
    assert_eq!(r.len(), 1);
    assert_eq!(wheel.get_item_timeout(&r[0].0), 86600000);
    

    //滚动 9006101 - 8660000 次， 只有时间为 90061010 毫秒的元素被取出（滚动到第 9006100 次）
    let temp = 90061010 / 10 - 8660000;
    println!("Temp {:?}", temp);
    for _i in 1..temp {
        now += wheel.frame_time;
        let r = roll(&mut wheel, now);
        if r.len() == 1 {
            println!("Temp {:?} - now {:?}", _i + 8660000, now);
        }
        assert_eq!(r.len(), 0);
    }
    now += wheel.frame_time;
    let r = roll(&mut wheel, now);
    assert_eq!(r.len(), 1);
    assert_eq!(wheel.get_item_timeout(&r[0].0), 90061010);

    println!("{:?}", wheel);
}

fn roll<T, const N1: usize, const N2: usize, const N3: usize, const N4: usize>(wheel: &mut LocalTimer<T, N1, N2, N3, N4>, now: u64) -> VecDeque<(TimeoutItem<T>, usize)> {
    let mut arr = VecDeque::new();

    loop {
        if let Some(item) = wheel.pop(now) {
            arr.push_front(item);
        }
        else {
            break;
        }
    }

    arr
}

fn pop_test<T, const N1: usize, const N2: usize, const N3: usize, const N4: usize>(wheel: &mut LocalTimer<T, N1, N2, N3, N4>, now: u64)-> Option<(TimeoutItem<T>, usize)> {
    wheel.pop(now)
}

#[test]
fn test1(){
    let mut wheel = LocalTimer::<i32, 100, 60, 60, 24>::new(10, run_millis());
    for i in 1..1001 {
        wheel.insert(i, 3000);
    }

    let mut count = 0;
    let mut roll_count = 0;
    let mut now = 0 as u64;

    roll::<i32, 100, 60, 60, 24>(&mut wheel, now);

    loop {
        // wheel.roll();
        now += wheel.frame_time;
        match pop_test(&mut wheel, now) {
            Some(_) => {
                count +=1;
                continue;
            },
            None => (),
        }
        roll_count += 1;
        if roll_count == 3002 {
            println!("count: {}", count);
            break;
        }
    }
}



#[test]
fn test_insert() {

    let mut wheel = LocalTimer::<i32, 1000, 60, 60, 24>::new(1, 0);

    let mut timer_refs = vec![];
    for i in 0..1000000 {
        timer_refs.push(wheel.insert(i, 10000 ));
    }

    let mut index = 0 as usize;

    for i in 0..10000 {
        let time = i;
        while wheel.check_sleep(time) == 0 {
            if let Some(_) = wheel.pop(time) {
                index += 1;
            }
        }
    }

    assert_eq!(timer_refs.len(), index);
}


#[test]
fn test_content() {

    let mut wheel = LocalTimer::<i32, 1000, 60, 60, 24>::new(1, 0);

    for i in 0..10000 {
        wheel.insert(i, 10000 );
    }

    for i in 0..10000 {
        let time = i;
        while wheel.check_sleep(time) == 0 {
            if let Some(item) = wheel.pop(time) {
                println!("{}", item.0.elem);
            }
        }
    }
}
