use std::io::Read as _;

fn main() {
	if std::env::var_os("VERBOSE").is_some() {
		simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default()).unwrap();
	}

	let action = std::env::args().nth(1).expect("need action");
	match action.as_str() {
		"parse" => parse(),
		"lex" => lex(),
		"decompose" => decompose(),
		_ => panic!("unknown action"),
	}
}

fn repl(mut callback: impl FnMut(&str)) {
	let mut input = String::new();
	if atty::is(atty::Stream::Stdin) {
		loop {
			eprint!("> ");
			input.clear();
			if std::io::stdin().read_line(&mut input).unwrap() == 0 {
				break;
			}
			callback(input.trim());
		}
	} else {
		std::io::stdin().read_to_string(&mut input).unwrap();
		callback(input.trim());
	}
}

fn decompose() {
	repl(|input| {
		let decomposed: Vec<_> = sneturfahi::decompose(input)
			.map(|span| span.slice(input).unwrap())
			.collect();
		println!("Decomposed: {decomposed:?}");
	});
}

fn parse() {
	repl(|input| {
		let lexed: Result<Vec<_>, _> = sneturfahi::lex(input).collect();
		let lexed = match lexed {
			Ok(lexed) => lexed,
			Err(error) => {
				println!("Lexing error: {error:?}");
				return;
			}
		};
		match sneturfahi::parse(&lexed) {
			Ok(ast) => println!("AST: {ast:#?}"),
			Err(error) => println!("Error: {error:?}"),
		}
	})
}

fn lex() {
	repl(|input| {
		let lexed: Result<Vec<_>, _> = sneturfahi::lex(input).collect();
		println!("Lexed: {lexed:?}");
	});
}
