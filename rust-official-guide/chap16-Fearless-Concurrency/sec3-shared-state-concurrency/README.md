# Similarities Between RefCell<T>/Rc<T> and Mutex<T>/Arc<T>

対比 [`RefCell<T>` / `Rc<T>`] と [`Mutex<T>` / `Arc<T>`]

> You might have noticed that counter is immutable but we could get a mutable reference to the value inside it; this means Mutex<T> provides interior mutability, as the Cell family does. In the same way we used RefCell<T> in Chapter 15 to allow us to mutate contents inside an Rc<T>, we use Mutex<T> to mutate contents inside an Arc<T>.
