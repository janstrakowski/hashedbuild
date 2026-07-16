/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

module.exports = grammar({
  name: "hashbuild",

  extras: $ => [
    $.comment,
    /\s/,
  ],

  rules: {
    comment: $ => /#[^\n]*/,
    identifier: $ => /[a-zA-Z_][a-zA-Z_0-9]*/,
    bool: $ => choice("false", "true"),
    integer: $ => /[0-9]*/,
    floatingp: $ => {
      const decimal_digit_seq = /[0-9]+/;
      const exponent = seq(/[eE]/, optional(/[-+]/), decimal_digit_seq);

      return token(choice(
        seq(optional(/[-+]/), optional(decimal_digit_seq), '.', decimal_digit_seq, optional(exponent)),
        seq(optional(/[-+]/), decimal_digit_seq, '.', optional(exponent)),
        seq(optional(/[-+]/), decimal_digit_seq, exponent),
        seq(optional(/[-+]/), /inf(inity)?/),
        /NaN/
      ));
    },
    path: $ => token(seq(
      choice('/', './', '../'),
      /[a-zA-Z0-9_\-\.\/]+/
    )),
    string: $ => token(seq(
      '"',
      repeat(choice(
        $._string_content,
        $.escape_sequence
      )),
      '"'
    )),
    _string_content: $ => token(prec(1, /[^\n"\\]+/)),
    escape_sequence: $ => token(seq(
      '\\',
      choice(
        /[0abfnrtv\\'"]/,
        /x[0-9a-fA-F]{2}/,
        /u[0-9a-fA-F]{4}/,
        /u\{[0-9a-fA-F]{1,6}\}/
      )
    )),
    structure: $ => seq(
      '{',
      repeat(seq($.structure_field, ",")),
      optional($.structure_field),
    ),
    structure_field: $ => seq(
      $.expression,
      optional(seq(
      "=",
      $.expression)),
    ),
    function: $ => seq(
      "|",
      $.identifier,
      "|",
      $.expression,
    ),
    add: $ => seq(
      $.expression,
      "+",
      $.expression,
    ),
    subtract: $ => seq(
      $.expression,
      "-",
      $.expression,
    ),
    multiply: $ => seq(
      $.expression,
      "*",
      $.expression,
    ),
    divide: $ => seq(
      $.expression,
      "/",
      $.expression,
    ),
    modulo: $ => seq(
      $.expression,
      "%",
      $.expression,
    ),
    not: $ => seq(
      "!",
      $.expression,
    ),
    or: $ => seq(
      "||",
      $.expression,
    ),
    and: $ => seq(
      "&&",
      $.expression,
    ),
    field_get: $ => seq(
      $.expression,
      ".",
      $.identifier,
    ),
    get: $ => seq(
      $.expression,
      "[",
      $.expression,
      "]",
    ),
    field_set: $ => seq(
      $.expression,
      ".+",
      $.identifier,
      $.expression,
    ),
    set: $ => seq(
      $.expression,
      "+[",
      $.expression,
      "]",
      $.expression,
    ),
    set_all: $ => seq(
      $.expression,
      "|<",
      $.expression,
    ),
    call: $ => seq(
      $.expression,
      $.expression,
    ),
    include: $ => seq(
      "include",
      $.expression,
    ),
    expression: $ => choice(
      $.identifier,
      $.bool,
      $.integer,
      $.floatingp,
      $.path,
      $.string,
      $.structure,
      $.function,
      $.add,
      $.subtract,
      $.multiply,
      $.divide,
      $.modulo,
      $.not,
      $.or,
      $.and,
      $.field_get,
      $.get,
      $.field_set,
      $.set,
      $.set_all,
      $.call,
      $.include,
    ),
    source_file: $ => $.expression,
  }
});
