use std::{vec::Vec, sync::{Arc,Weak,Mutex}};

pub(crate) struct Observable<T> {
    pub(crate) subscribers: Vec<Weak<Mutex<dyn Observer<T>>>>,
}

impl<T> Observable<T> {
    pub(crate) fn new() -> Observable<T> {
        Observable {
            subscribers: Vec::new(),
        }
    }
    pub(crate) fn register(&mut self, observer: Weak<Mutex<dyn Observer<T>>>) {
        self.subscribers.push(observer)
    }
    pub(crate) fn send_to_all(&self, message: &T) -> usize {
        (0..self.subscribers.len()).fold(0, |acc, i|
            match self.send_to(message, i) {
                Some(_) => acc+1,
                None => acc,
            }
        )
    }
    pub(crate) fn send_to(&self, message: &T, i: usize) -> Option<()> {
        self.subscribers.get(i)
            .and_then(|s|
                s.upgrade()
            ).and_then(|s| {
                s.lock().ok().as_mut().map(|s| {
                    s.notify(message);
                })
            })
    }
}

pub(crate) trait Observer<T> {
    fn notify(&mut self, event: &T);
}


#[cfg(test)]
mod tests {

    use crate::{*};

    #[derive(PartialEq, Debug)]
    enum MyMessage {
        Msg(&'static str),
    }

    #[derive(Default)]
    struct BeforeObserver {
        output: String,
        counter: usize,
    }

    impl Observer<MyMessage> for BeforeObserver {
        fn notify(&mut self, event: &MyMessage) {
            self.counter += 1;
            self.output = match event {
                MyMessage::Msg(str) => format!("{}, World", str),
            };
        }
    }

    #[derive(Default)]
    struct AfterObserver {
        output: String,
        counter: usize,
    }

    impl Observer<MyMessage> for AfterObserver {
        fn notify(&mut self, event: &MyMessage) {
            self.counter += 1;
            self.output = match event {
                MyMessage::Msg(str) => format!("Hello, {}", str),
            }
        }
    }

    #[test]
    fn test_observable() {
        let mut observable = Observable::<MyMessage>::new();

        let ob1: Arc<Mutex<BeforeObserver>> = Arc::new(Mutex::new(BeforeObserver::default()));

        // cannot directly cast type (requires unstable rust)
        // see https://github.com/rust-lang/rfcs/blob/master/text/0982-dst-coercion.md
        let ob1d: Arc<Mutex<dyn Observer<MyMessage>>> = ob1.clone();
        observable.register(Arc::downgrade(&ob1d));
        observable.send_to_all(&MyMessage::Msg("1"));
        
        {
            let lock1 = ob1.lock();
            let ob1 = lock1.as_ref().ok();
            assert_eq!(ob1.map(|v| v.counter), Some(1usize));
            assert_eq!(ob1.map(|v| &v.output[..]), Some(&format!("1, World")[..]));
        }

        let ob2: Arc<Mutex<AfterObserver>> = Arc::new(Mutex::new(AfterObserver::default()));
        let ob2d: Arc<Mutex<dyn Observer<MyMessage>>> = ob2.clone();
        observable.register(Arc::downgrade(&ob2d));
        observable.send_to_all(&MyMessage::Msg("2"));

        {
            let lock1 = ob1.lock();
            let ob1 = lock1.as_ref().ok();
            let lock2 = ob2.lock();
            let ob2 = lock2.as_ref().ok();
            assert_eq!(ob1.map(|v| v.counter), Some(2usize));
            assert_eq!(ob1.map(|v| &v.output[..]), Some(&format!("2, World")[..]));
            assert_eq!(ob2.map(|v| v.counter), Some(1usize));
            assert_eq!(ob2.map(|v| &v.output[..]), Some(&format!("Hello, 2")[..]));
        }

        observable.send_to(&MyMessage::Msg("3"), 1);

        {
            let lock1 = ob1.lock();
            let ob1 = lock1.as_ref().ok();
            let lock2 = ob2.lock();
            let ob2 = lock2.as_ref().ok();
            assert_eq!(ob1.map(|v| v.counter), Some(2usize));
            assert_eq!(ob1.map(|v| &v.output[..]), Some(&format!("2, World")[..]));
            assert_eq!(ob2.map(|v| v.counter), Some(2usize));
            assert_eq!(ob2.map(|v| &v.output[..]), Some(&format!("Hello, 3")[..]));
        }
    }

    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
