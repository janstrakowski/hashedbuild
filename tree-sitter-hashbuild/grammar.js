/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

module.exports = grammar({
  name: "hashbuild",

  // Precedence tiers, lowest to highest. Each binary/unary construct sits
  // between its own "*_construct" and "*_finalize" markers so that the
  // corresponding "_..._expression" rule can bottom out at the right level.
  precedences: _ => [
    [
      "language",
      "immediate_construct",
      "immediate_finalize",
      "accessible_construct",
      "accessible_finalize",
      "invocable_construct",
      "invocable_finalize",
      "code_accessible_construct",
      "code_accessible_finalize",
      "restructurable_construct",
      "restructurable_finalize",
      "multiplicable_construct",
      "multiplicable_finalize",
      "addable_construct",
      "addable_finalize",
      "comparable_construct",
      "comparable_finalize",
      "logical_construct",
      "logical_finalize",
      "general_construct",
      "general_finalize",
      "finalize",
    ]
  ],

  extras: $ => [
    $.comment,
    /\s/,
  ],

  rules: {
    // ------------------------------------------------------------------
    // Entry point
    // ------------------------------------------------------------------
    source_file: $ => $._expression,

    // ------------------------------------------------------------------
    // Trivia
    // ------------------------------------------------------------------
    comment: $ => /#[^\n]*/,

    // ------------------------------------------------------------------
    // Literals / atomic tokens
    // ------------------------------------------------------------------
    identifier: $ => /[a-zA-Z_][a-zA-Z_0-9]*/,
    bool: $ => choice("false", "true"),
    integer: $ => /[0-9]+/,
    float: $ => {
      const decimal_digit_seq = /[0-9]+/;
      const exponent = seq(/[eE]/, optional(/[-+]/), decimal_digit_seq);

      return token(prec(1, choice(
        seq(optional(decimal_digit_seq), '.', decimal_digit_seq, optional(exponent)),
        seq(decimal_digit_seq, '.', optional(exponent)),
        seq(decimal_digit_seq, exponent),
        /inf(inity)?/,
        /NaN/
      )));
    },
    path: $ => token(seq(
      choice('/', './', '../'),
      /[a-zA-Z0-9_\-\.\/]+/
    )),
    string: $ => seq(
      '"',
      repeat(choice(
        $._string_content,
        $.escape_sequence
      )),
      '"'
    ),
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

    // ------------------------------------------------------------------
    // Structures (record-like literals)
    // ------------------------------------------------------------------
    structure: $ => seq(
      '{',
      repeat(seq($.structure_field, ";")),
      optional($.structure_field),
      '}'
    ),
    structure_field: $ => choice(
      $._expression,
      seq($._expression, "=", $._expression),
      seq("=", $._expression),
    ),

    // ------------------------------------------------------------------
    // Patterns (used by functions and case matching)
    // ------------------------------------------------------------------
    _pattern: $ => seq("|", choice(
      $.identifier,
      $.structural_pattern,
      seq($.identifier, ":", $.structural_pattern),
    ), "|"),
    structural_pattern: $ => seq(
      "{",
      repeat(seq($.pattern_field, ",")),
      optional($.pattern_field),
      "}",
    ),
    pattern_field: $ => choice(
      $.identifier,
      $.structural_pattern,
      seq($.identifier, ":", $.structural_pattern),
    ),

    // ------------------------------------------------------------------
    // Functions
    // ------------------------------------------------------------------
    function: $ => seq(
      $._pattern,
      $._expression,
    ),

    // ------------------------------------------------------------------
    // Case / conditional expressions
    // ------------------------------------------------------------------
    if: $ => seq(
      "if",
      choice($._expression, $._pattern,),
      "then",
      $._expression,
      "else",
      $._expression,
    ),
    cases: $ => prec.right(seq(
      "cases",
      $._expression,
      $.case,
      repeat(seq(",", $.case)),
      optional(seq(",", $.default)),
      optional(","),
    )),
    case: $ => seq(
      "case",
      $._pattern,
      optional($._guard),
      $._expression,
    ),
    _guard: $ => prec("language", seq(
      seq("if", $._accessible_expression),
    )),
    default: $ => seq(
      "default",
      $._expression,
    ),

    // ------------------------------------------------------------------
    // Immediate: parenthesization
    // ------------------------------------------------------------------
    _braces: $ => prec("immediate_construct", seq(
      "(",
      $._expression,
      ")",
    )),

    // ------------------------------------------------------------------
    // Accessible: member/index access
    // ------------------------------------------------------------------
    field_get: $ => prec.left("accessible_construct", seq(
      $._accessible_expression,
      ".",
      $.identifier,
    )),
    get: $ => prec.left("accessible_construct", seq(
      $._accessible_expression,
      "[",
      $._expression,
      "]",
    )),

    // ------------------------------------------------------------------
    // Invocable: function calls
    // ------------------------------------------------------------------
    call: $ => prec.left("invocable_construct", seq(
      $._invocable_expression,
      $._invocable_expression,
    )),

    // ------------------------------------------------------------------
    // Code-accessible: includes
    // ------------------------------------------------------------------
    include: $ => prec.left("code_accessible_construct", seq(
      "include",
      $._code_accessible_expression,
    )),

    // ------------------------------------------------------------------
    // Restructurable: bulk structure assignment
    // ------------------------------------------------------------------
    set_all: $ => prec.left("restructurable_construct", seq(
      $._restructurable_expression,
      "|<",
      $._restructurable_expression,
    )),

    // ------------------------------------------------------------------
    // Multiplicable: * / %
    // ------------------------------------------------------------------
    multiply: $ => prec.left("multiplicable_construct", seq(
      $._multiplicable_expression,
      "*",
      $._multiplicable_expression,
    )),
    divide: $ => prec.left("multiplicable_construct", seq(
      $._multiplicable_expression,
      "/",
      $._multiplicable_expression,
    )),
    modulo: $ => prec.left("multiplicable_construct", seq(
      $._multiplicable_expression,
      "%",
      $._multiplicable_expression,
    )),

    // ------------------------------------------------------------------
    // Addable: + - (and unary negate)
    // ------------------------------------------------------------------
    add: $ => prec.left("addable_construct", seq(
      $._addable_expression,
      "+",
      $._addable_expression,
    )),
    subtract: $ => prec.left("addable_construct", seq(
      $._addable_expression,
      "-",
      $._addable_expression,
    )),
    negate: $ => prec.left("addable_construct", seq(
      "-",
      $._addable_expression,
    )),

    // ------------------------------------------------------------------
    // Comparable / logical: comparisons and boolean operators
    // ------------------------------------------------------------------
    greater_than: $ => prec("logical_construct", seq(
      $._comparable_expression,
      ">",
      $._comparable_expression,
    )),
    greater_than_or_equal: $ => prec("logical_construct", seq(
      $._comparable_expression,
      ">=",
      $._comparable_expression,
    )),
    equal: $ => prec("logical_construct", seq(
      $._comparable_expression,
      "==",
      $._comparable_expression,
    )),
    less_than: $ => prec("logical_construct", seq(
      $._comparable_expression,
      "<",
      $._comparable_expression,
    )),
    less_than_or_equal: $ => prec("logical_construct", seq(
      $._comparable_expression,
      "<=",
      $._comparable_expression,
    )),
    not: $ => prec.left(seq(
      "!",
      $._logical_expression,
    )),
    and: $ => prec.left("logical_construct", seq(
      $._logical_expression,
      "&&",
      $._logical_expression,
    )),
    or: $ => prec.left("logical_construct", seq(
      $._logical_expression,
      "||",
      $._logical_expression,
    )),

    // ------------------------------------------------------------------
    // General: argument piping
    // ------------------------------------------------------------------
    pass_as_argument: $ => prec.left("general_construct", seq(
      $._general_expression,
      "->",
      $._general_expression,
    )),

    // ------------------------------------------------------------------
    // Expression hierarchy (lowest tier first, each building on the last)
    // ------------------------------------------------------------------
    _immediate_expression: $ => prec("immediate_finalize", choice(
      $.identifier,
      $.bool,
      $.integer,
      $.float,
      $.path,
      $.string,
      $.structure,
      $._braces,
      $.function,
      $.cases,
      $.if,
    )),
    _accessible_expression: $ => prec("accessible_finalize", choice(
      $._immediate_expression,
      $.field_get,
      $.get,
    )),
    _invocable_expression: $ => prec("invocable_finalize", choice(
      $._accessible_expression,
      $.call,
    )),
    _code_accessible_expression: $ => prec("code_accessible_finalize", choice(
      $._invocable_expression,
      $.include,
    )),
    _restructurable_expression: $ => prec("restructurable_finalize", choice(
      $._code_accessible_expression,
      $.set_all,
    )),
    _multiplicable_expression: $ => prec("multiplicable_finalize", choice(
      $._restructurable_expression,
      $.multiply,
      $.divide,
      $.modulo,
    )),
    _addable_expression: $ => prec("addable_finalize", choice(
      $._multiplicable_expression,
      $.add,
      $.subtract,
      $.negate,
    )),
    _comparable_expression: $ => prec("comparable_finalize", choice(
      $._addable_expression,
    )),
    _logical_expression: $ => prec("logical_finalize", choice(
      $._comparable_expression,
      $.greater_than,
      $.greater_than_or_equal,
      $.equal,
      $.less_than_or_equal,
      $.less_than,
      $.not,
      $.or,
      $.and,
    )),
    _general_expression: $ => prec("general_finalize", choice(
      $._logical_expression,
      $.pass_as_argument,
    )),
    _expression: $ => prec("finalize", choice(
      $._general_expression,
    )),
  }
});
