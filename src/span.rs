pub type Location = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
	pub start: Location,
	pub end: Location,
}

impl Span {
	#[must_use]
	pub fn new(start: Location, end: Location) -> Self {
		assert!(end >= start);
		Self { start, end }
	}

	#[must_use]
	pub fn at(start: Location, len: u32) -> Self {
		Self {
			start,
			end: start + len,
		}
	}

	#[inline]
	#[must_use]
	pub fn len(self) -> u32 {
		self.end - self.start
	}

	#[inline]
	#[must_use]
	pub fn is_empty(self) -> bool {
		self.end == self.start
	}

	#[must_use]
	pub fn between(before: Self, after: Self) -> Option<Self> {
		if before.overlaps_with(after) || before.end >= after.start {
			None
		} else {
			Some(Self {
				start: before.end,
				end: after.start,
			})
		}
	}

	#[must_use]
	pub fn entire_slice(input: &str) -> Self {
		Self {
			start: 0,
			end: Location::try_from(input.len()).unwrap(),
		}
	}

	#[must_use]
	pub fn from_embedded_slice(outer_start_ptr: *const u8, embedded: &str) -> Self {
		let inner_start_ptr = embedded.as_ptr();
		assert!(inner_start_ptr >= outer_start_ptr);
		let start = Location::try_from(inner_start_ptr as usize - outer_start_ptr as usize).unwrap();
		let end = start + u32::try_from(embedded.len()).unwrap();
		Self { start, end }
	}

	#[must_use]
	pub fn slice(self, text: &str) -> Option<&str> {
		text.get(self.start as usize..self.end as usize)
	}

	#[must_use]
	pub fn slice_after(self, text: &str) -> Option<&str> {
		text.get(self.end as usize..)
	}

	#[must_use]
	pub fn slice_before(self, text: &str) -> Option<&str> {
		text.get(..self.start as usize)
	}

	#[must_use]
	pub fn contains(self, location: Location) -> bool {
		(self.start..self.end).contains(&location)
	}

	#[must_use]
	pub fn overlaps_with(self, other: Self) -> bool {
		fn non_commutative_helper(left: Span, right: Span) -> bool {
			// check for empty spans immediately before or after the other span
			if left.is_empty() && (right.end == left.start || right.start == left.start) {
				false
			} else {
				left.contains(right.start) || left.contains(right.end - 1)
			}
		}

		non_commutative_helper(self, other) || non_commutative_helper(other, self)
	}
}

#[test]
fn overlaps_with() {
	let span = Span::new(10, 20);
	// fully before
	assert_eq!(Span::new(0, 5).overlaps_with(span), false);
	// partially overlapping before
	assert_eq!(Span::new(5, 12).overlaps_with(span), true,);
	// fully within
	assert_eq!(Span::new(12, 18).overlaps_with(span), true);
	// empty within
	assert_eq!(Span::at(12, 0).overlaps_with(span), true);
	// partially overlapping after
	assert_eq!(Span::new(18, 22).overlaps_with(span), true);
	// fully after
	assert_eq!(Span::new(22, 24).overlaps_with(span), false);
	// back-to-back before
	assert_eq!(Span::new(0, 10).overlaps_with(span), false);
	// back-to-back after
	assert_eq!(Span::new(20, 30).overlaps_with(span), false);
	// empty immediately before
	assert_eq!(Span::at(10, 0).overlaps_with(span), false);
	// empty immediately after
	assert_eq!(Span::at(20, 0).overlaps_with(span), false);
}
