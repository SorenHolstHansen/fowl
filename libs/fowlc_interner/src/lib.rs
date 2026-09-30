use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::{LazyLock, RwLock};
use std::{collections::HashMap, hash::DefaultHasher};

static CURRENT_INTERNER: LazyLock<Interner> = Interner::new_locked();

#[derive(PartialEq, Eq)]
pub struct InternedStr(usize);

impl InternedStr {
    pub fn new(value: &str) -> InternedStr {
        CURRENT_INTERNER.intern(value)
    }
}

impl std::fmt::Debug for InternedStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", InternedStr::deref(self))
    }
}

impl Clone for InternedStr {
    fn clone(&self) -> Self {
        *self
    }
}

impl Copy for InternedStr {}

impl Deref for InternedStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        CURRENT_INTERNER.get(*self)
    }
}

struct Interner {
    container: RwLock<HashMap<usize, &'static str>>,
}

impl Interner {
    const fn new_locked() -> LazyLock<Interner> {
        LazyLock::new(|| Self {
            container: RwLock::new(HashMap::new()),
        })
    }

    fn intern(&self, value: &str) -> InternedStr {
        let mut container = self.container.write().unwrap();

        let mut s = DefaultHasher::new();
        value.hash(&mut s);
        let key = s.finish() as usize;

        if container.contains_key(&key) {
            return InternedStr(key);
        }

        // SAFETY:
        // We allocate the value on the heap and leak it, ensuring it remains valid
        // for the lifetime of the program.
        let leaked: &'static mut String = Box::leak(Box::new(value.to_string()));

        container.insert(key, leaked);

        InternedStr(key)
    }

    fn get(&self, interned: InternedStr) -> &'static str {
        let container = self.container.read().unwrap();
        let Some(value) = container.get(&interned.0) else {
            panic!("interned index value is invalid: {}", interned.0)
        };

        value
    }
}
