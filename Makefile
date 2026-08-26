lexer_gen:
	re2rust libs/lexer/src/lexing.re \
		--output libs/lexer/src/lexing.rs \
		--no-unsafe \
		--start-conditions \
		--no-generation-date \
		--no-version

fuzz: fuzz_build fuzz_run
	
fuzz_build:
	cargo afl build -p fowl-fuzz --release
	
fuzz_run:
	cargo afl fuzz \
		-i fuzz/corpus \
		-o fuzz/artifacts \
		-x fuzz/lang.dict \
		target/release/fowl-fuzz
	
	
