struct Lock<'a> {
    _marker: std::marker::PhantomData<fn(&'a ()) -> &'a ()>,
}

impl<'a> Lock<'a> {
    fn resource(&self) -> Resource<'a> {
        Resource {
            _marker: std::marker::PhantomData,
            amount: 42,
        }
    }
}

struct Resource<'a> {
    _marker: std::marker::PhantomData<fn(&'a ()) -> &'a ()>,
    amount: u32,
}

impl<'a> Resource<'a> {
    const fn amount(&self, lock: &'a Lock<'a>) -> u32 {
        self.amount
    }
}

fn main() {
    let lock1 = Lock {
        _marker: std::marker::PhantomData,
    };
    let lock2 = Lock {
        _marker: std::marker::PhantomData,
    };

    let resource1 = lock1.resource();
    let resource2 = lock2.resource();

    let amount1 = resource1.amount(&lock2);
}
