struct UnorderedInner<'l, T> {
    inner: Option<Pin<Box<dyn Future<Output = T> + 'l>>>,
}
impl<'l, T> Stream for UnorderedInner<'l, T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match &mut self.inner {
            Some(i) => match i.poll(cx) {
                Poll::Ready(f) => {
                    self.inner = None;
                    Poll::Ready(Some(f))
                }
                Poll::Pending => Poll::Pending,
            },
            None => Poll::Ready(None),
        }
    }
}
pub struct Unordered<'l, T> {
    _inner: futures_concurrency::vec::Merge<UnorderedInner<'l, T>>,
}

#[allow(clippy::type_complexity)]
struct Scope<'env, 'scope, T> {
    inner: RefCell<Vec<Box<dyn FnMut() -> Pin<Box<dyn Future<Output = T> + 'env>> + 'env>>>,
    scope: PhantomData<&'scope mut &'scope ()>,
    env: PhantomData<&'env mut &'env ()>,
}

impl<'env, 'scope, T> Default for Scope<'env, 'scope, T> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            scope: Default::default(),
            env: Default::default(),
        }
    }
}
impl<'scope, 'env, T> Scope<'scope, 'env, T> {
    pub fn add<F, R>(&'env self, mut func: F)
    where
        F: FnMut() -> R + 'scope,
        R: Future<Output = T> + 'scope,
    {
        self.inner
            .borrow_mut()
            .push(Box::new(move || Box::pin(func())));
    }
}
fn scope<'env, T, F: for<'scope> FnOnce(&'scope Scope<'env, 'scope, T>)>(func: F) {
    let mut i = Scope::default();
    func(&mut i);
}
