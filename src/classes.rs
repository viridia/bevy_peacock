use bevy::ecs::system::EntityCommands;
use bevy::prelude::*;
// use bevy::utils::all_tuples;
use bevy::utils::HashSet;
use impl_trait_for_tuples::*;

/// List of class names which are attached to a given UiNode. Style selectors can use these
/// class names to conditionally apply styles.
#[derive(Component, Default)]
pub struct ElementClasses(pub HashSet<String>);

impl ElementClasses {
    /// Construct a new [`ElementClasses`] component from a tuple of class names.
    pub fn new<'a, C: ClassNames<'a>>(classes: C) -> Self {
        Self(classes.to_set())
    }

    /// Add a classname to this element. Be careful using this method with `.class_names()`,
    /// because the latter will overwrite any changes you make with this method.
    pub fn add_class(&mut self, cls: &str) {
        self.0.insert(cls.to_string());
    }

    /// Remove a classname from this element. Be careful using this method with `.class_names()`,
    /// because the latter will overwrite any changes you make with this method.
    pub fn remove_class(&mut self, cls: &str) {
        self.0.remove(cls);
    }
}

pub struct ConditionalClassNames<'a, C: ClassNames<'a>> {
    pub(crate) inner: C,
    pub(crate) enabled: bool,
    pub(crate) marker: std::marker::PhantomData<&'a ()>,
}

impl<'a, C: ClassNames<'a> + PartialEq> PartialEq for ConditionalClassNames<'a, C> {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner && self.enabled == other.enabled && self.marker == other.marker
    }
}

/// Tuple of class names, possibly conditional, possibly nested.
pub trait ClassNames<'a>: Send + Clone {
    /// Return the number of class names.
    fn len(&self) -> usize;

    /// True if the list of class names is empty.
    fn is_empty(&self) -> bool;

    /// Add all of the enabled class names to a set.
    fn add_classes(&self, classes: &mut HashSet<String>);

    /// Make this set of class names conditional; if the condition is false, then the
    /// class names will not be added to the set.
    fn if_true(self, enabled: bool) -> ConditionalClassNames<'a, Self>
    where
        Self: Sized,
    {
        ConditionalClassNames {
            inner: self,
            enabled,
            marker: std::marker::PhantomData,
        }
    }

    /// Convert this set of class names into a HashSet.
    fn to_set(&self) -> HashSet<String>
    where
        Self: Sized,
    {
        let mut result = HashSet::<String>::with_capacity(self.len());
        self.add_classes(&mut result);
        result
    }
}

impl<'a> ClassNames<'a> for () {
    fn len(&self) -> usize {
        0
    }

    fn is_empty(&self) -> bool {
        true
    }

    fn add_classes(&self, _classes: &mut HashSet<String>) {}
}

impl<'a> ClassNames<'a> for String {
    fn len(&self) -> usize {
        1
    }

    fn is_empty(&self) -> bool {
        false
    }

    fn add_classes(&self, classes: &mut HashSet<String>) {
        classes.insert(self.clone());
    }
}

impl<'a> ClassNames<'a> for &str {
    fn len(&self) -> usize {
        1
    }

    fn is_empty(&self) -> bool {
        false
    }

    fn add_classes(&self, classes: &mut HashSet<String>) {
        classes.insert(self.to_string());
    }
}

impl<'a, C: ClassNames<'a>> ClassNames<'a> for ConditionalClassNames<'a, C> {
    fn len(&self) -> usize {
        if self.enabled {
            self.inner.len()
        } else {
            0
        }
    }

    fn is_empty(&self) -> bool {
        if self.enabled {
            self.inner.is_empty()
        } else {
            true
        }
    }

    fn add_classes(&self, classes: &mut HashSet<String>) {
        if self.enabled {
            self.inner.add_classes(classes);
        }
    }
}

impl<'a, C: ClassNames<'a>> Clone for ConditionalClassNames<'a, C> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            enabled: self.enabled,
            marker: self.marker,
        }
    }
}

// macro_rules! impl_class_names {
//     ($($T:ident),*) => {
//         impl<$($T: ConditionalClassName),*> ClassNamesTuple for ($($T,)*) {
//             fn to_vec(self) -> Vec<Option<String>> {
//                 Vec::from([$(self.$T.to_class()),* ])
//             }
//         }
//     };
// }

// all_tuples!(impl_class_names, 1, 16, S);

#[impl_for_tuples(1, 16)]
impl<'a> ClassNames<'a> for Tuple {
    for_tuples!( where #( Tuple: ClassNames<'a> )* );

    fn len(&self) -> usize {
        for_tuples!( #( self.Tuple.len() )+* );
    }

    fn is_empty(&self) -> bool {
        false
    }

    fn add_classes(&self, classes: &mut HashSet<String>) {
        for_tuples!( #(
            self.Tuple.add_classes(classes);
        )* );
    }
}

/// Trait that permits adding a tuple of [`ClassName`]s to an object.
pub trait WithClasses {
    /// Add a tuple of [`StyleHandle`]s to the object.
    fn class_names<'a, C: ClassNames<'a>>(&mut self, classes: C) -> &mut Self;
}

/// Insert class names using commands.
impl<'w, 's, 'a> WithClasses for EntityCommands<'w, 's, 'a> {
    fn class_names<'c, C: ClassNames<'c>>(&mut self, classes: C) -> &mut Self {
        self.insert(ElementClasses(classes.to_set()));
        self
    }
}

/// Update or insert class names into an [`EntityWorldMut`].
impl<'w> WithClasses for EntityWorldMut<'w> {
    fn class_names<'a, C: ClassNames<'a>>(&mut self, classes: C) -> &mut Self {
        if let Some(mut ec) = self.get_mut::<ElementClasses>() {
            ec.0 = classes.to_set();
        } else {
            self.insert(ElementClasses(classes.to_set()));
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use bevy::utils::hashbrown::HashSet;

    use super::*;

    fn get_names<'a, CN: ClassNames<'a>>(class_names: CN) -> HashSet<String> {
        let mut classes = HashSet::with_capacity(class_names.len());
        class_names.add_classes(&mut classes);
        classes
    }

    #[test]
    fn test_class_names() {
        let cl = get_names(());
        assert_eq!(cl, HashSet::new());

        let cl = get_names("test");
        assert_eq!(cl, ["test".to_owned()].into());

        let cl = get_names(("one", "two"));
        assert_eq!(cl, ["one".to_owned(), "two".to_owned()].into());

        let cl = get_names(("one".if_true(true), "two"));
        assert_eq!(cl, ["one".to_owned(), "two".to_owned()].into());

        let cl = get_names(("one".if_true(false), "two"));
        assert_eq!(cl, ["two".to_owned()].into());

        let cl = get_names(("one".if_true(true).if_true(false), "two"));
        assert_eq!(cl, ["two".to_owned()].into());
    }
}
