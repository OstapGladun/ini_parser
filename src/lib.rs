use pest::Parser;
use pest::iterators::Pair;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum IniError {
    #[error("Помилка парсингу: {0}")]
    ParseError(#[from] pest::error::Error<Rule>),
    #[error("Помилка читання файлу: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Внутрішня помилка: невірне правило {0:?}")]
    InvalidRule(Rule),
    #[error("Неможливо розпарсити значення: {0}")]
    ValueParseError(String),
}

/// Документація для `docs.rs`
///
/// Основний парсер INI, що використовує граматику з `src/grammar.pest`.
///
/// Цей парсер ідентифікує наступні правила граматики:
/// - `file`: Корінь документа.
/// - `section`: Секція у форматі `[section.name]`.
/// - `pair`: Пара `key.name=value` (з підтримкою inline-коментарів).
/// - `value`: Типізоване значення (`string_quoted`, `number`, `boolean`, `string_simple`).
/// - `LINE_COMMENT`: Коментар на весь рядок (`;` або `#`).
#[derive(pest_derive::Parser)]
#[grammar = "grammar.pest"]
pub struct IniParser;

#[derive(Debug, Clone, PartialEq)]
pub enum IniValue {
    String(String),
    Number(f64),
    Boolean(bool),
}

pub type IniData = HashMap<String, HashMap<String, IniValue>>;

/// Парсить вхідний рядок INI у структурований `IniData`.
pub fn parse_ini(input: &str) -> Result<IniData, IniError> {
    let file_pair = IniParser::parse(Rule::file, input)?
        .next()
        .ok_or(IniError::InvalidRule(Rule::file))?;

    let mut data: IniData = HashMap::new();
    let mut current_section_name = String::from("");
    data.insert(current_section_name.clone(), HashMap::new());

    for pair in file_pair.into_inner() {
        match pair.as_rule() {
            Rule::line => {
                let line_content = pair.into_inner().next();

                if let Some(line_pair) = line_content {
                    match line_pair.as_rule() {
                        Rule::section => {
                            current_section_name = line_pair
                                .into_inner()
                                .next()
                                .ok_or(IniError::InvalidRule(Rule::section))?
                                .as_str()
                                .to_string();

                            data.entry(current_section_name.clone()).or_default();
                        }
                        Rule::pair => {
                            let mut inner = line_pair.into_inner();

                            let key = inner
                                .next()
                                .ok_or(IniError::InvalidRule(Rule::pair))?
                                .as_str()
                                .to_string();

                            let value_pair =
                                inner.next().ok_or(IniError::InvalidRule(Rule::pair))?;

                            let value = parse_value_pair(value_pair)?;

                            if let Some(section_map) = data.get_mut(&current_section_name) {
                                section_map.insert(key, value);
                            }
                        }
                        Rule::LINE_COMMENT | Rule::NEWLINE => (),

                        _ => return Err(IniError::InvalidRule(line_pair.as_rule())),
                    }
                }
            }
            Rule::EOI => (),

            _ => return Err(IniError::InvalidRule(pair.as_rule())),
        }
    }

    Ok(data)
}

/// Допоміжна функція для парсингу `value` в `IniValue`.
fn parse_value_pair(value_pair: Pair<Rule>) -> Result<IniValue, IniError> {
    match value_pair.as_rule() {
        Rule::value => {
            let inner_value = value_pair
                .into_inner()
                .next()
                .ok_or(IniError::InvalidRule(Rule::value))?;
            let inner_str = inner_value.as_str();

            match inner_value.as_rule() {
                Rule::string_quoted => {
                    let unquoted = &inner_str[1..inner_str.len() - 1];
                    Ok(IniValue::String(unquoted.to_string()))
                }
                Rule::number => {
                    let num = inner_str
                        .parse::<f64>()
                        .map_err(|e| IniError::ValueParseError(e.to_string()))?;
                    Ok(IniValue::Number(num))
                }
                Rule::boolean => {
                    let bool_val = match inner_str {
                        "true" | "yes" => true,
                        "false" | "no" => false,
                        _ => {
                            return Err(IniError::ValueParseError(format!(
                                "Invalid boolean: {}",
                                inner_str
                            )));
                        }
                    };
                    Ok(IniValue::Boolean(bool_val))
                }
                Rule::string_simple => Ok(IniValue::String(inner_str.trim().to_string())),
                _ => Err(IniError::InvalidRule(inner_value.as_rule())),
            }
        }
        _ => Err(IniError::InvalidRule(value_pair.as_rule())),
    }
}
