use std::cmp::Ordering;


#[derive(Clone, Debug)]
/// 延时任务数据结构
pub struct TimeoutItem<T> {
    /// 数据
    pub elem: T,
    /// 延时 - 外部不可用
    frame_point: u64
}

impl<T> TimeoutItem<T>{
    pub fn new(elem: T, time_point: u64) -> TimeoutItem<T> {
        TimeoutItem{
            elem,
            frame_point: time_point
        }
    }
    pub fn get_frame_point(&self) -> u64 {
        self.frame_point
    }
}

impl<T> Ord for TimeoutItem<T> {
    fn cmp(&self, other: &TimeoutItem<T>) -> Ordering {
        self.frame_point.cmp(&other.frame_point)
    }
}

impl<T> PartialOrd for TimeoutItem<T> {
    fn partial_cmp(&self, other: &TimeoutItem<T>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> PartialEq for TimeoutItem<T> {
    fn eq(&self, other: &TimeoutItem<T>) -> bool {
        self.frame_point == other.frame_point
    }
}

impl<T> Eq for TimeoutItem<T> {
}
