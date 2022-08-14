fn main() {
	if std::env::var_os("VERBOSE").is_some() {
		simplelog::SimpleLogger::init(log::LevelFilter::Trace, simplelog::Config::default()).unwrap();
	}

	let action = std::env::args().nth(1).expect("need action");
	match action.as_str() {
		"parse" => parse(),
		"lex" => lex(),
		"decompose" => decompose(),
		"decompose-stdin" => {
			let mut input = String::new();
			std::io::Read::read_to_string(&mut std::io::stdin().lock(), &mut input).unwrap();
			let decomposed: Vec<_> = sneturfahi::decompose(&input).collect();
			println!("Decomposed: {decomposed:?}");
		}
		_ => panic!("unknown action"),
	}
}

fn repl(mut callback: impl FnMut(&str)) {
	let mut input = String::new();
	loop {
		eprint!("> ");
		input.clear();
		if std::io::stdin().read_line(&mut input).unwrap() == 0 {
			break;
		}
		callback(input.trim());
	}
}

fn decompose() {
	repl(|input| {
		let decomposed: Vec<_> = sneturfahi::decompose(input).collect();
		println!("Decomposed: {decomposed:?}");
	})
}

fn parse() {
	repl(|input| {
		let lexed = sneturfahi::lex(input);
		match sneturfahi::parse(lexed.into_iter()) {
			Ok(ast) => println!("AST: {ast:#?}"),
			Err(error) => println!("Error: {error:?}"),
		}
	})
}

fn lex() {
	repl(|input| println!("Lexed: {:?}", sneturfahi::lex(input.trim())));
}
