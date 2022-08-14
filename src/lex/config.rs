#[derive(Debug, Default, Clone, Copy)]
#[non_exhaustive]
pub struct Config {
	pub zoi_delimiter_comparison: ZoiDelimiterComparisonStrategy,
}

/// How to compare zoi delimiters
#[derive(Debug, Clone, Copy)]
pub struct ZoiDelimiterComparisonStrategy {
	pub ignore_commas_in_start: bool,
	pub ignore_commas_in_end: bool,
}

impl Default for ZoiDelimiterComparisonStrategy {
	fn default() -> Self {
		Self {
			ignore_commas_in_start: true,
			ignore_commas_in_end: true,
		}
	}
}

impl ZoiDelimiterComparisonStrategy {
	pub(in crate::lex) fn compare(self, start: &str, end: &str) -> bool {
		fn not_comma(&ch: &char) -> bool {
			ch != ','
		}

		macro_rules! filter {
			($s:expr) => {
				$s.chars().filter(not_comma)
			};
		}

		match (self.ignore_commas_in_start, self.ignore_commas_in_end) {
			(true, true) => filter!(start).eq(filter!(end)),
			(true, false) => filter!(start).eq(end.chars()),
			(false, true) => start.chars().eq(filter!(end)),
			(false, false) => start == end,
		}
	}
}
