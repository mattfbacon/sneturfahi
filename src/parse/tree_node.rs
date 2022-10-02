use crate::span::Location;

/// An abstraction over nodes in the CST.
pub trait TreeNode {
	/// Get the name of the node.
	#[must_use]
	fn name(&self) -> &'static str;

	/// Determine whether the node contains any experimental cmavo at any level.
	#[must_use]
	fn experimental(&self) -> bool;

	/// Get the starting location of the node, or `None` if the node is empty.
	///
	/// `start_location` and `end_location` should both return Some or both return None.
	#[must_use]
	fn start_location(&self) -> Option<Location>;

	/// Get the ending location of the node, or `None` if the node is empty.
	///
	/// `start_location` and `end_location` should both return Some or both return None.
	#[must_use]
	fn end_location(&self) -> Option<Location>;

	/// Run the passed closure for each child of the node.
	fn for_each_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn TreeNode));
}

impl TreeNode for crate::lex::Token {
	fn name(&self) -> &'static str {
		self.selmaho.as_repr()
	}

	fn experimental(&self) -> bool {
		self.experimental
	}

	fn start_location(&self) -> Option<Location> {
		Some(self.span.start)
	}

	fn end_location(&self) -> Option<Location> {
		Some(self.span.end)
	}

	fn for_each_child<'a>(&'a self, _: &mut dyn FnMut(&'a dyn TreeNode)) {}
}

impl<T: TreeNode> TreeNode for &T {
	fn name(&self) -> &'static str {
		T::name(self)
	}

	fn experimental(&self) -> bool {
		T::experimental(self)
	}

	fn start_location(&self) -> Option<Location> {
		T::start_location(self)
	}

	fn end_location(&self) -> Option<Location> {
		T::end_location(self)
	}

	fn for_each_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn TreeNode)) {
		T::for_each_child(self, f);
	}
}

impl<T: TreeNode> TreeNode for Box<T> {
	fn name(&self) -> &'static str {
		T::name(self)
	}

	fn experimental(&self) -> bool {
		T::experimental(self)
	}

	fn start_location(&self) -> Option<Location> {
		T::start_location(self)
	}

	fn end_location(&self) -> Option<Location> {
		T::end_location(self)
	}

	fn for_each_child<'a>(&'a self, f: &mut dyn FnMut(&'a dyn TreeNode)) {
		T::for_each_child(self, f);
	}
}

pub(in crate::parse) trait TreeNodeChild {
	fn invoke_with_self<'a>(&'a self, f: &mut dyn FnMut(&'a dyn TreeNode));
	fn experimental(&self) -> bool;
	fn start_location(&self) -> Option<Location>;
	fn end_location(&self) -> Option<Location>;
}

impl<T: TreeNode> TreeNodeChild for T {
	fn invoke_with_self<'a>(&'a self, f: &mut dyn FnMut(&'a dyn TreeNode)) {
		f(self);
	}

	fn experimental(&self) -> bool {
		<Self as TreeNode>::experimental(self)
	}

	fn start_location(&self) -> Option<Location> {
		<Self as TreeNode>::start_location(self)
	}

	fn end_location(&self) -> Option<Location> {
		<Self as TreeNode>::end_location(self)
	}
}

macro_rules! box_impl {
	(@actual ($($generics:ident),*), $actual:ty, $ty:ty) => {
		impl<$($generics: TreeNodeChild),*> TreeNodeChild for $ty {
			fn invoke_with_self<'a>(&'a self, f: &mut dyn FnMut(&'a dyn TreeNode)) {
				<$actual>::invoke_with_self(self, f)
			}

			fn experimental(&self) -> bool {
				<$actual>::experimental(self)
			}

			fn start_location(&self) -> Option<Location> {
				<$actual>::start_location(self)
			}

			fn end_location(&self) -> Option<Location> {
				<$actual>::end_location(self)
			}
		}
	};
	(($($generics:ident),*) => $ty:ty) => {
		box_impl!(@actual ($($generics),*), $ty, Box<$ty>);
		box_impl!(@actual ($($generics),*), $ty, &'_ $ty);
	};
}
pub(in crate::parse) use box_impl;

macro_rules! iterator_impls {
	($ty:ty) => {
		impl<T: TreeNodeChild> TreeNodeChild for $ty {
			fn invoke_with_self<'a>(&'a self, f: &mut dyn FnMut(&'a dyn TreeNode)) {
				for child in self.iter() {
					child.invoke_with_self(f);
				}
			}

			fn experimental(&self) -> bool {
				self.iter().any(|item| item.experimental())
			}

			fn start_location(&self) -> Option<Location> {
				self.iter().find_map(|child| child.start_location())
			}

			fn end_location(&self) -> Option<Location> {
				self.iter().rev().find_map(|child| child.end_location())
			}
		}
		box_impl!((T) => $ty);
	};
	($($ty:ty),+ $(,)?) => {
		$(iterator_impls!($ty);)+
	}
}

iterator_impls![Option<T>, Box<[T]>, &[T]];

macro_rules! tuple_impls {
	// base case
	() => {};
	(@single $($idents:ident),*) => {
		#[allow(non_snake_case)]
		impl<$($idents: TreeNodeChild),*> TreeNodeChild for ($($idents,)*) {
			fn invoke_with_self<'a>(&'a self, f: &mut dyn FnMut(&'a dyn TreeNode)) {
				let ($($idents,)*) = self;
				$($idents.invoke_with_self(f);)*
			}

			fn experimental(&self) -> bool {
				let ($($idents,)*) = self;
				false $(|| $idents.experimental())*
			}

			fn start_location(&self) -> Option<Location> {
				let ($($idents,)*) = self;
				$(
					if let Some(location) = $idents.start_location() {
						return Some(location);
					}
				)*
				None
			}

			fn end_location(&self) -> Option<Location> {
				let ($($idents,)*) = self;
				let children = [$($idents as &dyn TreeNodeChild,)*];
				children.iter().rev().find_map(|child| child.end_location())
			}
		}

		box_impl!(($($idents),*) => ($($idents,)*));
	};
	($first:ident $(, $idents:ident)*) => {
		tuple_impls!(@single $first $(, $idents)*);
		tuple_impls!($($idents),*);
	};
}

tuple_impls![T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15];
