pub struct Dependency{
        name: String,
    }

    impl Dependency{
        pub fn new(name: String) -> Dependency {
            Dependency{
                name
            }
        }
    }