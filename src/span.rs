/// A location within the input.
/// This is a `u32` rather than a `usize` since we don't really need the capacity that `usize` would provide.
pub type Location = u32;

/// A region within the input, similar to [std::ops::Range] but less awful.
/// It can be resolved to a subslice of the input, but also provides other useful utilities.
#[cfg_attr(
	target_pointer_width = "64",
	doc = "It is also smaller than a string slice, though it requires the original input to get an actual `&str` substring."
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
	/// The index of the first byte of the span. This is included in the span.
	pub start: Location,
	/// The index immediately after the last byte of the span. This is not included in the span.
	pub end: Location,
}

impl Span {
	/// Create a new span that starts at `start` and ends at `end`.
	///
	/// # Examples
	///
	/// ```rust
	/// # use sneturfahi::span::Span;
	/// let span = Span::new(1, 3);
	/// assert_eq!(span.len(), 2);
	/// assert_eq!(span.slice("abcdef").unwrap(), "bc");
	/// ```
	#[must_use]
	pub fn new(start: Location, end: Location) -> Self {
		assert!(end >= start);
		Self { start, end }
	}

	/// Create a new span of `input` that starts at `start` has the length `len`.
	/// Returns `None` if the resulting span would not be within the input.
	///
	/// # Examples
	///
	/// ```rust
	/// # use sneturfahi::span::Span;
	/// let span = Span::at(2, 2);
	/// assert_eq!(span.len(), 2);
	/// assert_eq!(span.slice("abcdef").unwrap(), "cd");
	/// ```
	#[must_use]
	pub fn at(start: Location, len: u32) -> Self {
		Self {
			start,
			end: start + len,
		}
	}

	/// Get the length of the span, that is, the distance between the start and the end.
	///
	/// # Examples
	///
	/// ```rust
	/// # use sneturfahi::span::Span;
	/// let input = "abcdef";
	/// assert_eq!(Span::new(1, 3).len(), 2);
	/// assert_eq!(Span::at(1, 3).len(), 3);
	/// ```
	#[inline]
	#[must_use]
	pub fn len(self) -> u32 {
		self.end - self.start
	}

	/// `self.len() == 0`
	///
	/// # Examples
	///
	/// ```rust
	/// # use sneturfahi::span::Span;
	/// assert_eq!(Span::new(10, 20).is_empty(), false);
	/// assert_eq!(Span::new(10, 10).is_empty(), true);
	/// assert_eq!(Span::at(5, 0).is_empty(), true);
	/// ```
	#[inline]
	#[must_use]
	pub fn is_empty(self) -> bool {
		self.end == self.start
	}

	/// Get the span between `before` and `after`.
	/// Returns `None` if the spans overlap or if `after` is before `before`.
	///
	/// # Examples
	///
	/// ```rust
	/// # use sneturfahi::span::Span;
	/// let input = "abcdefghi";
	/// let before_span = Span::at(0, 3);
	/// let after_span = Span::at(6, 3);
	/// let between_span = Span::between(before_span, after_span).unwrap();
	/// assert_eq!(between_span, Span::at(3, 3));
	/// assert_eq!(between_span.slice(input).unwrap(), "def");
	/// ```
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

	/// Create a span that covers all of `input`.
	/// Equivalent to `Span::new(input, 0, u32::try_from(input.len()).unwrap())`.
	///
	/// # Examples
	///
	/// ```rust
	/// # use sneturfahi::span::Span;
	/// let input = "abcdef";
	/// let span = Span::entire_slice(input);
	/// assert_eq!(span.len(), input.len() as u32);
	/// assert_eq!(span.slice(input).unwrap(), input);
	/// ```
	#[must_use]
	pub fn entire_slice(input: &str) -> Self {
		Self {
			start: 0,
			end: Location::try_from(input.len()).unwrap(),
		}
	}

	/// Create a span that covers `embedded` based on its position relative to `outer_start_ptr`.
	/// Returns `None` if `outer_start_ptr` is after the start of `embedded`.
	/// The intended use is to store a pointer to the start of the input, and then use this function to get spans while processing embedded slices of that input.
	///
	/// # Examples
	///
	/// ```rust
	/// # use sneturfahi::span::Span;
	/// let input = "abc def ghi";
	/// let input_start = input.as_ptr();
	/// let split: Vec<_> = input
	/// 	.split_whitespace()
	/// 	.map(|chunk| Span::from_embedded_slice(input_start, chunk))
	/// 	.collect();
	/// assert_eq!(split, [Span::at(0, 3), Span::at(4, 3), Span::at(8, 3)]);
	/// ```
	#[must_use]
	pub fn from_embedded_slice(outer_start_ptr: *const u8, embedded: &str) -> Self {
		let inner_start_ptr = embedded.as_ptr();
		assert!(inner_start_ptr >= outer_start_ptr);
		let start = Location::try_from(inner_start_ptr as usize - outer_start_ptr as usize).unwrap();
		let end = start + u32::try_from(embedded.len()).unwrap();
		Self { start, end }
	}

	/// Get the substring of `text` that this span delineates.
	/// Returns `None` if the span is out of the bounds of `text`.
	///
	/// # Examples
	///
	/// ```rust
	/// # use sneturfahi::span::Span;
	/// let input = "abcdef";
	/// let span = Span::at(1, 3);
	/// assert_eq!(span.slice(input).unwrap(), "bcd");
	/// ```
	///
	/// The API does not prevent slicing unrelated strings, but it is essentially a logical error, unless you know that the span is also valid for that string.
	/// The result may be `None` or not, regardless of whether `text` is the intended source of this span:
	///
	/// ```rust
	/// # use sneturfahi::span::Span;
	/// # let span = Span::at(1, 3);
	/// let unrelated_string = "gg";
	/// assert!(span.slice(unrelated_string).is_none());
	/// let unrelated_string_that_happens_to_be_long_enough = "zyxwvutsr";
	/// assert_eq!(
	/// 	span
	/// 		.slice(unrelated_string_that_happens_to_be_long_enough)
	/// 		.unwrap(),
	/// 	"yxw"
	/// );
	/// ```
	#[must_use]
	pub fn slice(self, text: &str) -> Option<&str> {
		text.get(self.start as usize..self.end as usize)
	}

	/// Get the substring of `text` that is after this span.
	/// Returns `None` if the end of the span is out of the bounds of `text`.
	/// As with `slice`, it is typically a logic error to slice an unrelated string, and is *not* guaranteed to return `None`.
	///
	/// # Examples
	///
	/// Typical usage:
	///
	/// ```rust
	/// # use sneturfahi::span::Span;
	/// let input = "abcdef";
	/// let within_span = Span::at(2, 2);
	/// assert_eq!(within_span.slice_after(input), Some("ef"));
	/// ```
	///
	/// An out-of-bounds span:
	///
	/// ```
	/// # use sneturfahi::span::Span;
	/// # let input = "abcdef";
	/// let out_of_bounds_span = Span::at(5, 3);
	/// assert_eq!(out_of_bounds_span.slice(input), None);
	/// ```
	#[must_use]
	pub fn slice_after(self, text: &str) -> Option<&str> {
		text.get(self.end as usize..)
	}

	/// Get the substring of `text` that is before this span.
	/// Returns `None` if the beginning of the span is out of the bounds of `text`.
	/// As with `slice`, it is typically a logic error to slice an unrelated string, and is *not* guaranteed to return `None`.
	///
	/// # Examples
	///
	/// Typical usage:
	///
	/// ```rust
	/// # use sneturfahi::span::Span;
	/// let input = "abcdef";
	/// let within_span = Span::at(2, 3);
	/// assert_eq!(within_span.slice_before(input), Some("ab"));
	/// ```
	///
	/// An out-of-bounds span:
	///
	/// ```
	/// # use sneturfahi::span::Span;
	/// # let input = "abcdef";
	/// let out_of_bounds_span = Span::at(7, 3);
	/// assert_eq!(out_of_bounds_span.slice_before(input), None);
	/// ```
	#[must_use]
	pub fn slice_before(self, text: &str) -> Option<&str> {
		text.get(..self.start as usize)
	}

	/// Checks if a location is contained within the span.
	/// Spans are inclusive of the start but not the end.
	///
	/// # Examples
	///
	/// ```rust
	/// # use sneturfahi::span::Span;
	/// let span = Span::new(3, 6);
	/// assert!(!span.contains(2));
	/// assert!(span.contains(3));
	/// assert!(span.contains(4));
	/// assert!(span.contains(5));
	/// assert!(!span.contains(6));
	/// ```
	#[must_use]
	pub fn contains(self, location: Location) -> bool {
		(self.start..self.end).contains(&location)
	}

	/// Checks if two spans overlap.
	/// This operation is commutative.
	///
	/// # Examples
	///
	/// ```rust
	/// # use sneturfahi::span::Span;
	/// assert_eq!(Span::new(10, 30).overlaps_with(Span::new(20, 40)), true);
	/// assert_eq!(Span::new(0, 10).overlaps_with(Span::new(20, 30)), false);
	/// ```
	///
	/// Empty spans immediately before or after the other span are not considered to overlap:
	///
	/// ```rust
	/// # use sneturfahi::span::Span;
	/// assert_eq!(Span::at(10, 0).overlaps_with(Span::new(10, 20)), false);
	/// assert_eq!(Span::at(20, 0).overlaps_with(Span::new(10, 20)), false);
	/// ```
	#[must_use]
	pub fn overlaps_with(self, other: Self) -> bool {
		fn non_commutative_empty_check(left: Span, right: Span) -> bool {
			left.is_empty() && (right.end == left.start || right.start == left.start)
		}
		fn non_commutative_helper(left: Span, right: Span) -> bool {
			left.contains(right.start) || left.contains(right.end - 1)
		}

		if non_commutative_empty_check(self, other) || non_commutative_empty_check(other, self) {
			return false;
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
