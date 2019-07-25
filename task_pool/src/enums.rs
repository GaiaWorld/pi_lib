
#[derive(Debug, Clone)]
pub enum DequeStat{
    Normal, //任务的队列
    Locked, //被锁的任务队列
    HalfLocked, //被锁的任务队列, 一旦向其中push任务， 就会解锁
}

#[derive(Debug)]
pub enum FreeSign {
    Success,
    Error,
    Ignore,
}

#[derive(Debug)]
pub enum TaskType {
    DynSync = 1,
    StaticSync = 2,
    DynAsync = 4,
    StaticAsync = 8,
}