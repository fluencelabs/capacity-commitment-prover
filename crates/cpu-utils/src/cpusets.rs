use std::str::FromStr;

use ccp_shared::types::CPUIdType;
use ccp_shared::types::LogicalCoreId;

pub const LINUX_CPUSET_CPUS_PATH: &str = "/sys/fs/cgroup/cpuset.cpus";

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum CpuListParserError {
    #[error("invalid CPU list {list:?}: {error}")]
    InvalidNumber {
        list: String,
        error: <CPUIdType as FromStr>::Err,
    },
}

pub fn read_cpu_list_string() -> std::io::Result<String> {
    std::fs::read_to_string(LINUX_CPUSET_CPUS_PATH)
}

pub fn parse_cpu_list(content: &str) -> Result<Vec<LogicalCoreId>, CpuListParserError> {
    if content.is_empty() {
        return Ok(vec![]);
    }
    let ranges = content
        .split(',')
        .map(|group| match group.split_once('-') {
            Some((a, b)) => a
                .trim()
                .parse::<CPUIdType>()
                .and_then(|a| b.trim().parse().map(move |b| (a..=b))),
            None => group.trim().parse().map(|a| (a..=a)),
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| CpuListParserError::InvalidNumber {
            list: content.to_owned(),
            error,
        })?;

    let mut cpu_ids: Vec<_> = ranges
        .into_iter()
        .flatten()
        .map(LogicalCoreId::new)
        .collect();

    // Make output consistent and remove possible duplicates.
    cpu_ids.sort_unstable();
    cpu_ids.dedup();

    Ok(cpu_ids)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser() {
        let list = parse_cpu_list("4,0-2,6-8,1").unwrap();
        assert_eq!(
            list,
            vec![
                LogicalCoreId::new(0),
                LogicalCoreId::new(1),
                LogicalCoreId::new(2),
                LogicalCoreId::new(4),
                LogicalCoreId::new(6),
                LogicalCoreId::new(7),
                LogicalCoreId::new(8),
            ]
        );
    }

    #[test]
    fn test_parser_spaces() {
        let list = parse_cpu_list(" 4 , 0 - 2 , 6-8, 1  ").unwrap();
        assert_eq!(
            list,
            vec![
                LogicalCoreId::new(0),
                LogicalCoreId::new(1),
                LogicalCoreId::new(2),
                LogicalCoreId::new(4),
                LogicalCoreId::new(6),
                LogicalCoreId::new(7),
                LogicalCoreId::new(8),
            ]
        );
    }

    #[test]
    fn test_empty() {
        let list = parse_cpu_list("").unwrap();
        assert!(list.is_empty());
    }

    #[test]
    fn test_invalid_1() {
        let content = "1-a";
        let err = parse_cpu_list(content).unwrap_err();
        let expected_err = "a".parse::<CPUIdType>().unwrap_err();
        assert_eq!(
            err,
            CpuListParserError::InvalidNumber {
                list: content.to_owned(),
                error: expected_err
            }
        );
    }

    #[test]
    fn test_invalid_empty_range_1() {
        let content = "3,";
        let err = parse_cpu_list(content).unwrap_err();
        let expected_err = "".parse::<CPUIdType>().unwrap_err();
        assert_eq!(
            err,
            CpuListParserError::InvalidNumber {
                list: content.to_owned(),
                error: expected_err
            }
        );
    }

    #[test]
    fn test_invalid_empty_range_2() {
        let content = ",3";
        let err = parse_cpu_list(content).unwrap_err();
        let expected_err = "".parse::<CPUIdType>().unwrap_err();
        assert_eq!(
            err,
            CpuListParserError::InvalidNumber {
                list: content.to_owned(),
                error: expected_err
            }
        );
    }
}
