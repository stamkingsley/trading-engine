use crossbeam_queue::{ArrayQueue, SegQueue};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct LockFreeQueue<T> {
    bounded_queue: Option<Arc<ArrayQueue<T>>>,
    unbounded_queue: Option<Arc<SegQueue<T>>>,
    enqueue_count: Arc<AtomicUsize>,
    dequeue_count: Arc<AtomicUsize>,
}

impl<T> Clone for LockFreeQueue<T> {
    fn clone(&self) -> Self {
        Self {
            bounded_queue: self.bounded_queue.clone(),
            unbounded_queue: self.unbounded_queue.clone(),
            enqueue_count: self.enqueue_count.clone(),
            dequeue_count: self.dequeue_count.clone(),
        }
    }
}

impl<T> LockFreeQueue<T> {
    /// 创建有界队列
    pub fn bounded(capacity: usize) -> Self {
        Self {
            bounded_queue: Some(Arc::new(ArrayQueue::new(capacity))),
            unbounded_queue: None,
            enqueue_count: Arc::new(AtomicUsize::new(0)),
            dequeue_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// 创建无界队列
    pub fn unbounded() -> Self {
        Self {
            bounded_queue: None,
            unbounded_queue: Some(Arc::new(SegQueue::new())),
            enqueue_count: Arc::new(AtomicUsize::new(0)),
            dequeue_count: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// 入队操作
    pub fn enqueue(&self, item: T) -> Result<(), T> {
        let result = if let Some(ref bounded) = self.bounded_queue {
            bounded.push(item)
        } else if let Some(ref unbounded) = self.unbounded_queue {
            unbounded.push(item);
            Ok(())
        } else {
            unreachable!()
        };

        if result.is_ok() {
            self.enqueue_count.fetch_add(1, Ordering::Relaxed);
        }
        result
    }

    /// 出队操作
    pub fn dequeue(&self) -> Option<T> {
        let result = if let Some(ref bounded) = self.bounded_queue {
            bounded.pop()
        } else if let Some(ref unbounded) = self.unbounded_queue {
            unbounded.pop()
        } else {
            unreachable!()
        };

        if result.is_some() {
            self.dequeue_count.fetch_add(1, Ordering::Relaxed);
        }
        result
    }

    /// 获取队列长度（近似值）
    pub fn len(&self) -> usize {
        if let Some(ref bounded) = self.bounded_queue {
            bounded.len()
        } else if let Some(ref unbounded) = self.unbounded_queue {
            unbounded.len()
        } else {
            0
        }
    }

    /// 检查队列是否为空
    pub fn is_empty(&self) -> bool {
        if let Some(ref bounded) = self.bounded_queue {
            bounded.is_empty()
        } else if let Some(ref unbounded) = self.unbounded_queue {
            unbounded.is_empty()
        } else {
            true
        }
    }

    /// 获取入队计数
    pub fn enqueue_count(&self) -> usize {
        self.enqueue_count.load(Ordering::Relaxed)
    }

    /// 获取出队计数
    pub fn dequeue_count(&self) -> usize {
        self.dequeue_count.load(Ordering::Relaxed)
    }
}

unsafe impl<T: Send> Send for LockFreeQueue<T> {}
unsafe impl<T: Send> Sync for LockFreeQueue<T> {}