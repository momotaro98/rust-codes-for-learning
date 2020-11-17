## Object Safety Is Required for Trait Objects

> You can only make object-safe traits into trait objects. Some complex rules govern all the properties that make a trait object safe, but in practice, only two rules are relevant. A trait is object safe if all the methods defined in the trait have the following properties:

* The return type isn’t Self.
* There are no generic type parameters.
