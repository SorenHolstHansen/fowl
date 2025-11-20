/*!re2c
    re2c:encoding:utf8 = 1;
    re2c:encoding-policy = ignore;
 */

use super::token::{Token, TokenKind};
use super::lexer_error::{LexerError, LexerErrorKind};
use super::lexer::Lexer;

#[allow(unused_braces)]
#[rustfmt::skip]
impl<'src> Iterator for Lexer<'src> {
    type Item = Result<Token<'src>, LexerError<'src>>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.peeked.take() {
            return Some(next);
        }

        if self.eof { return None }

        self.token = self.cursor;

        /*!re2c
        re2c:api                     = generic;
        re2c:yyfill:enable           = 0;
        re2c:define:YYCTYPE          = u8;
        re2c:define:YYLESSTHAN       = "self.input.len() - self.cursor < @@{len}";
        re2c:define:YYPEEK           = "*self.input.as_bytes().get(self.cursor).unwrap_or(&0)";
        re2c:define:YYSKIP           = "self.cursor += 1;";
        re2c:define:YYBACKUP         = "self.marker = self.cursor;";
        re2c:define:YYRESTORE        = "self.cursor = self.marker;";
        re2c:define:YYSHIFT          = "self.cursor += @@{shift};";
        re2c:define:YYGETCONDITION   = "self.cond";
        re2c:define:YYSETCONDITION   = "self.cond = @@{cond};";
        re2c:eof                     = 0;
        
        // Keywords
        <INIT> "fn"                    { return self.token(TokenKind::Fn) }
        <INIT> "let"                   { return self.token(TokenKind::Let) }
        <INIT> "return"                { return self.token(TokenKind::Return) }
        <INIT> "if"                    { return self.token(TokenKind::If) }
        <INIT> "else"                  { return self.token(TokenKind::Else) }
        <INIT> "for"                   { return self.token(TokenKind::For) }
        <INIT> "break"                 { return self.token(TokenKind::Break) }
        <INIT> "continue"              { return self.token(TokenKind::Continue) }
        <INIT> "in"                    { return self.token(TokenKind::In) }
        <INIT> "use"                   { return self.token(TokenKind::Use) }
        <INIT> "pub"                   { return self.token(TokenKind::Pub) }
        <INIT> "match"                 { return self.token(TokenKind::Match) }
        <INIT> "none"                  { return self.token(TokenKind::None) }
        <INIT> "try"                   { return self.token(TokenKind::Try) }
        <INIT> "catch"                 { return self.token(TokenKind::Catch) }
        <INIT> "throw"                 { return self.token(TokenKind::Throw) }
        <INIT> "struct"                { return self.token(TokenKind::Struct) }
        <INIT> "enum"                  { return self.token(TokenKind::Enum) }
        <INIT> "and"                   { return self.token(TokenKind::And) }
        <INIT> "or"                    { return self.token(TokenKind::Or) }
        <INIT> "mut"                   { return self.token(TokenKind::Mut) }

        // Types
		<INIT> "int"                   { return self.token(TokenKind::Int) }
		<INIT> "float"                 { return self.token(TokenKind::Float) }
		<INIT> "string"                { return self.token(TokenKind::String) }
		<INIT> "bool"                  { return self.token(TokenKind::Bool) }
		<INIT> "void"                  { return self.token(TokenKind::Void) }

        // Operators
        <INIT> "="                     { return self.token(TokenKind::Eq) }
        <INIT> "=="                    { return self.token(TokenKind::EqEq) }
        <INIT> "!="                    { return self.token(TokenKind::Neq) }
        <INIT> "<"                     { return self.token(TokenKind::Lt) }
        <INIT> ">"                     { return self.token(TokenKind::Gt) }
        <INIT> "<="                    { return self.token(TokenKind::LtEq) }
        <INIT> ">="                    { return self.token(TokenKind::GtEq) }
        <INIT> "+"                     { return self.token(TokenKind::Plus) }
        <INIT> "-"                     { return self.token(TokenKind::Minus) }
        <INIT> "*"                     { return self.token(TokenKind::Star) }
        <INIT> "**"                    { return self.token(TokenKind::StarStar) }
        <INIT> "/"                     { return self.token(TokenKind::Slash) }
        <INIT> "%"                     { return self.token(TokenKind::Percent) }
        <INIT> "!"                     { return self.token(TokenKind::Bang) }

        // Assignment operators
        <INIT> "+="                    { return self.token(TokenKind::PlusEq) }
        <INIT> "-="                    { return self.token(TokenKind::MinusEq) }
        <INIT> "*="                    { return self.token(TokenKind::StarEq) }
        <INIT> "/="                    { return self.token(TokenKind::SlashEq) }

        // Literals
        <INIT> "true"                  { return self.token(TokenKind::BoolLiteral(true)) }
        <INIT> "false"                 { return self.token(TokenKind::BoolLiteral(false)) }
        <INIT> [+-]?[0-9]+             { return self.int() }
        <INIT> [+-]?[0-9]+ "." [0-9]*  { return self.float() }

        // Strings
        <INIT> "\""                    => STRING { return self.token(TokenKind::StringInterpolationStart); }
        <STRING> [^"\\{]+              { return self.token(TokenKind::StringLiteral(self.token_text())) }
        <STRING> "\\" .                { return self.token(TokenKind::StringLiteral(&self.input[(self.token + 1)..(self.token + 2)])); }
        <STRING> "{"                   => INIT { self.interpolation_depth += 1; return self.token(TokenKind::LBrace) }
        // string end
        <STRING> "\""                  => INIT { return self.token(TokenKind::StringInterpolationEnd) }
        

        // Identifiers
        <INIT> [a-zA-Z_] [a-zA-Z_0-9]* { return self.ident() }

        // Structural
        <INIT> ":"                     { return self.token(TokenKind::Colon) }
        <INIT> ";"                     { return self.token(TokenKind::Semicolon) }
        <INIT> "("                     { return self.token(TokenKind::LParen) }
        <INIT> ")"                     { return self.token(TokenKind::RParen) }
        <INIT> "{"                     { return self.token(TokenKind::LBrace) }
        <INIT> "}"                     { if self.interpolation_depth > 0 { self.cond = YYC_STRING; self.interpolation_depth -= 1; }; return self.token(TokenKind::RBrace) }
        <INIT> "["                     { return self.token(TokenKind::LBracket) }
        <INIT> "]"                     { return self.token(TokenKind::RBracket) }
        <INIT> ","                     { return self.token(TokenKind::Comma) }
        <INIT> "."                     { return self.token(TokenKind::Dot) }
        <INIT> "?"                     { return self.token(TokenKind::QuestionMark) }

        // Line comments
        <INIT> "//"[^\x00\n]*          { return self.next() }

        // Whitespace
        <INIT> [ \t\v\f]+              { return self.next() }
        <INIT> "\n"                    { return self.next() }

        // EOF
        <INIT, STRING> $                { self.eof = true; return self.token(TokenKind::Eof) }

        // Anything else
        <INIT, STRING> *                       { return Some(Err(LexerError { span: self.span(), kind: LexerErrorKind::UnexpectedToken(self.token_text()) })) }

        */
    }
}

/*!conditions:re2c
format = "pub const @@{cond}: usize = @@{num};\n";
*/
