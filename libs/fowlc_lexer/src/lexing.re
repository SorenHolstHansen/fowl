/*!re2c
    re2c:encoding:utf8 = 1;
    re2c:encoding-policy = ignore;
 */

/*!include:re2c "unicode_categories.re" */

#![allow(clippy::all)]
use super::token::{Token, TokenKind};
use super::lexer_error::{LexerError, UnexpectedCharacter, UnmatchedInterpolation, MissingBrace};
use super::lexer::Lexer;

#[allow(unused_braces)]
#[rustfmt::skip]
impl<'src> Lexer<'src> {
    pub fn next(&mut self) -> Result<Token<'src>, LexerError<'src>>  {
        self.next_internal(true)
    }

    pub(crate) fn next_internal(&mut self, use_peek_queue: bool) -> Result<Token<'src>, LexerError<'src>> {
        if use_peek_queue {
            if let Some(next) = self.peek_queue.pop_front() {
                return next;
            }
        }
        if let Some(forced) = self.force_next_token.take() {
            return Ok(forced);
        }

        if self.eof { return self.error(LexerError::EofAlreadyReached) }

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

        id_start    = L | Nl | [$_];
        id_continue = id_start | Mn | Mc | Nd | Pc | [\u200D\u05F3];
        identifier  = id_start id_continue*;
        
        // Keywords
        <INIT> "fn"                    { return self.token(TokenKind::Fn) }
        <INIT> "let"                   { return self.token(TokenKind::Let) }
        <INIT> "return"                { return self.token(TokenKind::Return) }
        <INIT> "if"                    { return self.token(TokenKind::If) }
        <INIT> "for"                   { return self.token(TokenKind::For) }
        <INIT> "break"                 { return self.token(TokenKind::Break) }
        <INIT> "continue"              { return self.token(TokenKind::Continue) }
        <INIT> "in"                    { return self.token(TokenKind::In) }
        <INIT> "is"                    { return self.token(TokenKind::Is) }
        <INIT> "use"                   { return self.token(TokenKind::Use) }
        <INIT> "public"                { return self.token(TokenKind::Public) }
        <INIT> "internal"              { return self.token(TokenKind::Internal) }
        <INIT> "private"               { return self.token(TokenKind::Private) }
        <INIT> "none"                  { return self.token(TokenKind::None) }
        <INIT> "try"                   { return self.token(TokenKind::Try) }
        <INIT> "catch"                 { return self.token(TokenKind::Catch) }
        <INIT> "throw"                 { return self.token(TokenKind::Throw) }
        <INIT> "struct"                { return self.token(TokenKind::Struct) }
        <INIT> "enum"                  { return self.token(TokenKind::Enum) }
        <INIT> "and"                   { return self.token(TokenKind::And) }
        <INIT> "or"                    { return self.token(TokenKind::Or) }
        <INIT> "mut"                   { return self.token(TokenKind::Mut) }
        <INIT> "on"                    { return self.token(TokenKind::On) }
        <INIT> "self"                    { return self.token(TokenKind::Self_) }
        <INIT> "impl"                    { return self.token(TokenKind::Impl) }

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
        <INIT> "true"                  { return self.token(TokenKind::BoolLiteral) }
        <INIT> "false"                 { return self.token(TokenKind::BoolLiteral) }
        <INIT> [+-]?[0-9][0-9_]*             { return self.token(TokenKind::IntLiteral) }
        <INIT> [+-]?[0-9][0-9_]* "." [0-9]+  { return self.token(TokenKind::FloatLiteral) }

        // Strings
        <INIT> "\""                    => STRING { if self.interpolation_depth > 0 {
                                                return self.error(LexerError::UnmatchedInterpolation(UnmatchedInterpolation {span: self.span(), missing: MissingBrace::Right}))
                                            } else {
                                                return self.token(TokenKind::StringInterpolationStart);
                                            }
                                       }
        <STRING> "}"                   { if self.interpolation_depth > 0 {
                                            self.interpolation_depth -= 1;
                                            self.cond = YYC_STRING;
                                            return self.token(TokenKind::RBrace)
                                        } else {
                                            return self.error(LexerError::UnmatchedInterpolation(UnmatchedInterpolation {span: self.span(), missing: MissingBrace::Left}))
                                        }
                                       }
        <STRING> [^"\\{\\}]+           { return self.token(TokenKind::StringLiteral) }
        <STRING> "\\" .                { return self.token(TokenKind::StringLiteral); }
        <STRING> "{"                   => INIT { self.interpolation_depth += 1; return self.token(TokenKind::LBrace) }
        // string end
        <STRING> "\""                  => INIT { return self.token(TokenKind::StringInterpolationEnd) }
        

        <INIT> "_"                     { return self.token(TokenKind::Underscore) }

        // Identifiers
        <INIT> identifier              { return self.token(TokenKind::Ident) }

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

        // Line comments
        <INIT> "//"[^\x00\n]*          { return self.token(TokenKind::Comment) }

        // Whitespace
        <INIT> [ \t\v\f\n]+              { return self.token(TokenKind::Whitespace) }

        // EOF
        <INIT, STRING> $               { self.eof = true; return self.token(TokenKind::Eof) }

        // Anything else
        <INIT, STRING> *               { self.find_boundary(); return self.error(LexerError::UnexpectedCharacter(UnexpectedCharacter {span: self.span(), char: self.token_text()})) }

        */
    }
}

/*!conditions:re2c
format = "pub const @@{cond}: usize = @@{num};\n";
*/
