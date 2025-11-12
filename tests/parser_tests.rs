use anyhow::Result;
use ini_parser::{IniValue, Rule, parse_ini};
use pest::Parser;

#[test]
fn test_full_parsing() -> Result<()> {
    let input = r#"
; Глобальні налаштування
global_bool = yes
api.key = "my-secret-key" # inline comment

[server.prod]
port = 8080
active = true
load_factor = -0.75
url = "https://api.example.com"
empty_quoted = ""

[server.dev]
port = 3000
active = false
url = http://localhost:3000 ; simple string
"#;

    let data = parse_ini(input)?;

    let global = data.get("").unwrap();
    assert_eq!(global.get("global_bool").unwrap(), &IniValue::Boolean(true));
    assert_eq!(
        global.get("api.key").unwrap(),
        &IniValue::String("my-secret-key".to_string())
    );

    let server_prod = data.get("server.prod").unwrap();
    assert_eq!(server_prod.get("port").unwrap(), &IniValue::Number(8080.0));
    assert_eq!(server_prod.get("active").unwrap(), &IniValue::Boolean(true));
    assert_eq!(
        server_prod.get("load_factor").unwrap(),
        &IniValue::Number(-0.75)
    );
    assert_eq!(
        server_prod.get("url").unwrap(),
        &IniValue::String("https://api.example.com".to_string())
    );
    assert_eq!(
        server_prod.get("empty_quoted").unwrap(),
        &IniValue::String("".to_string())
    );

    let server_dev = data.get("server.dev").unwrap();
    assert_eq!(server_dev.get("port").unwrap(), &IniValue::Number(3000.0));
    assert_eq!(server_dev.get("active").unwrap(), &IniValue::Boolean(false));
    assert_eq!(
        server_dev.get("url").unwrap(),
        &IniValue::String("http://localhost:3000".to_string())
    );

    Ok(())
}

#[test]
fn test_whitespace_and_empty_lines() -> Result<()> {
    let input = r#"
key1=value1

[section1]
key2  =  value2


    key3 = value3

"#;
    let data = parse_ini(input)?;
    let global = data.get("").unwrap();
    let section1 = data.get("section1").unwrap();

    assert_eq!(
        global.get("key1").unwrap(),
        &IniValue::String("value1".to_string())
    );
    assert_eq!(
        section1.get("key2").unwrap(),
        &IniValue::String("value2".to_string())
    );
    assert_eq!(
        section1.get("key3").unwrap(),
        &IniValue::String("value3".to_string())
    );
    assert_eq!(global.len(), 1);
    assert_eq!(section1.len(), 2);
    Ok(())
}

#[test]
fn test_all_value_types() -> Result<()> {
    let input = r#"
b_true = true
b_false = false
b_yes = yes
b_no = no
n_int = 123
n_neg = -456
n_float = 123.456
n_float_pos = +1.2
s_simple =   simple string with spaces   
"#;
    let data = parse_ini(input)?.get("").unwrap().clone();

    assert_eq!(data.get("b_true").unwrap(), &IniValue::Boolean(true));
    assert_eq!(data.get("b_false").unwrap(), &IniValue::Boolean(false));
    assert_eq!(data.get("b_yes").unwrap(), &IniValue::Boolean(true));
    assert_eq!(data.get("b_no").unwrap(), &IniValue::Boolean(false));
    assert_eq!(data.get("n_int").unwrap(), &IniValue::Number(123.0));
    assert_eq!(data.get("n_neg").unwrap(), &IniValue::Number(-456.0));
    assert_eq!(data.get("n_float").unwrap(), &IniValue::Number(123.456));
    assert_eq!(data.get("n_float_pos").unwrap(), &IniValue::Number(1.2));
    assert_eq!(
        data.get("s_simple").unwrap(),
        &IniValue::String("simple string with spaces".to_string())
    );
    Ok(())
}

#[test]
fn test_rule_dotted_identifier() -> Result<()> {
    let mut pairs = ini_parser::IniParser::parse(Rule::dotted_identifier, "a.b.c_123-d")?;
    assert_eq!(pairs.next().unwrap().as_str(), "a.b.c_123-d");
    Ok(())
}

#[test]
fn test_rule_pair_with_inline_comment() -> Result<()> {
    let mut pairs = ini_parser::IniParser::parse(Rule::pair, "key = value ; comment")?;
    let mut inner = pairs.next().unwrap().into_inner();

    let key_rule = inner.next().unwrap();
    assert_eq!(key_rule.as_rule(), Rule::dotted_identifier);
    assert_eq!(key_rule.as_str(), "key");

    let value_rule = inner.next().unwrap();
    assert_eq!(value_rule.as_rule(), Rule::value);

    let simple_string_rule = value_rule.into_inner().next().unwrap();
    assert_eq!(simple_string_rule.as_rule(), Rule::string_simple);
    assert_eq!(simple_string_rule.as_str(), "value ");

    assert!(
        inner.next().is_none(),
        "INLINE_COMMENT має бути тихим і не з'являтися в парі"
    );

    Ok(())
}

#[test]
fn test_fail_unclosed_section() {
    let input = "[section";
    assert!(parse_ini(input).is_err());
}

#[test]
fn test_fail_unclosed_quote() {
    let input = "key = \"hello";
    assert!(parse_ini(input).is_err());
}

#[test]
fn test_fail_invalid_key() {
    let input = "!key = value";
    assert!(parse_ini(input).is_err());
}

#[test]
fn test_fail_missing_equals() {
    let input = "key value";
    assert!(parse_ini(input).is_err());
}

#[test]
fn test_fail_no_value() {
    let input = "key =";
    assert!(parse_ini(input).is_err());
}

#[test]
fn test_fail_invalid_boolean() {
    let input = "key = truthy";
    let data = parse_ini(input).unwrap();
    assert_eq!(
        data.get("").unwrap().get("key").unwrap(),
        &IniValue::String("truthy".to_string())
    );
}
