pub struct Dependency{
        pub name: String,
        pub deps: Vec<String>
    }

    
    impl Dependency{
        pub fn new(name: String) -> Dependency {
            Dependency{
                name, deps: Vec::new()
            }
        }
    }