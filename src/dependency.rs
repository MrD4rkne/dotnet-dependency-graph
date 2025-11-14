pub struct Dependency{
        pub name: String,
    }

    
    impl Dependency{
        pub fn new(name: String) -> Dependency {
            Dependency{
                name
            }
        }
    }