#![deny(
	absolute_paths_not_starting_with_crate,
	elided_lifetimes_in_paths,
	explicit_outlives_requirements,
	future_incompatible,
	keyword_idents,
	macro_use_extern_crate,
	meta_variable_misuse,
	missing_abi,
	non_ascii_idents,
	nonstandard_style,
	noop_method_call,
	pointer_structural_match,
	private_in_public,
	rust_2018_idioms
)]
#![warn(clippy::pedantic, unused_qualifications)]
#![forbid(unsafe_code)]

use std::io::Read as _;

use sneturfahi::parse::tree_node::TreeNode;

/// Interact with sneturfahi via the command line.
#[derive(argh::FromArgs, Debug)]
#[allow(clippy::struct_excessive_bools)] // CLI arguments
struct Args {
	/// enable verbose logging
	#[argh(switch)]
	verbose: bool,
	/// print the decomposed words
	#[argh(switch)]
	decompose: bool,
	/// print the lexed tokens
	#[argh(switch)]
	lex: bool,
	/// print the CST
	#[argh(switch)]
	cst: bool,
	/// collapse single-child nodes in CST
	#[argh(switch)]
	collapse_cst: bool,
}

fn main() {
	let args: Args = argh::from_env();

	if args.verbose {
		simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default()).unwrap();
	}

	if !(args.decompose || args.lex || args.cst) {
		eprintln!("At least one of `--decompose`, `--lex`, `--cst` is required.");
		return;
	}

	repl(move |input| {
		if args.decompose {
			let decomposed = sneturfahi::decompose(input);
			println!("Decomposed: {:?}", DebugWithIterator(decomposed, input));
		}

		let lexed: Result<Vec<_>, _> = sneturfahi::lex(input).collect();

		let lexed = match lexed {
			Ok(lexed) => lexed,
			Err(error) => {
				println!("Lexing error: {error:?}");
				return;
			}
		};

		if args.lex {
			println!(
				"Lexed: {:?}",
				DebugWithIterator(lexed.iter().copied(), input)
			);
		}

		if args.cst {
			let arena = sneturfahi::Arena::new();
			match sneturfahi::Cst::parse(&lexed, &arena) {
				Ok(cst) => print_tree_node(cst.root(), input, args.collapse_cst),
				Err(error) => {
					if let Some(any_token) = error
						.location
						.first()
						.filter(|token| token.selmaho == sneturfahi::lex::Selmaho::AnyText)
					{
						println!(
							"The token {any_token:?} could not be lexed. The content of this token is {:?}.",
							any_token.span.slice(input).unwrap()
						);
					} else {
						println!("Parse error: {error:?}");
					}
				}
			}
			println!("size of CST arena: {}", arena.allocated_bytes());
		}
	});
}

fn print_tree_node(root: &dyn TreeNode, input: &str, collapse: bool) {
	fn inner(mut node: &dyn TreeNode, input: &str, collapse: bool, level: usize) {
		if collapse {
			while {
				let mut num_children = 0;
				node.for_each_child(&mut |_| {
					num_children += 1;
				});
				num_children == 1
			} {
				node.for_each_child(&mut |child| {
					node = child;
				});
			}
		}

		print!(
			"{blank:level$}{}{}",
			node.name(),
			if node.experimental() { "*" } else { "" },
			blank = ""
		);
		let span = node
			.start_location()
			.map(|start| sneturfahi::Span::new(start, node.end_location().unwrap()));
		if let Some(span) = span {
			print!(" @ {span:?} {:?}", span.slice(input).unwrap());
		}
		println!();

		node.for_each_child(&mut move |child| inner(child, input, collapse, level + 2));
	}

	inner(root, input, collapse, 0);
}

#[derive(Clone, Copy)]
struct WithInput<'input, T>(T, &'input str);

impl std::fmt::Debug for WithInput<'_, sneturfahi::Span> {
	fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			formatter,
			"{:?} @ {:?}",
			self.0.slice(self.1).unwrap(),
			self.0.start..self.0.end
		)
	}
}

impl std::fmt::Debug for WithInput<'_, sneturfahi::lex::Token> {
	fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			formatter,
			"{}{}({:?})",
			self.0.selmaho.as_repr(),
			if self.0.experimental { "*" } else { "" },
			WithInput(self.0.span, self.1)
		)
	}
}

struct DebugWithIterator<'input, T>(T, &'input str);

impl<'input, T> std::fmt::Debug for DebugWithIterator<'input, T>
where
	T: Iterator + Clone,
	WithInput<'input, <T as Iterator>::Item>: std::fmt::Debug,
{
	fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		formatter
			.debug_list()
			.entries(self.0.clone().map(move |item| WithInput(item, self.1)))
			.finish()
	}
}

fn repl(mut callback: impl FnMut(&str)) {
	use std::io::BufRead as _;

	let mut input = String::new();
	let mut stdin = std::io::stdin().lock();
	if atty::is(atty::Stream::Stdin) {
		loop {
			eprint!("> ");
			input.clear();
			if stdin.read_line(&mut input).unwrap() == 0 {
				break;
			}
			callback(input.trim());
		}
	} else {
		stdin.read_to_string(&mut input).unwrap();
		callback(input.trim());
	}
}
