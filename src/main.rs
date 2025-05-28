use sha256::hash;

fn main() {
    let input = "abcd";
    println!("{}",hash(input));
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_hash_empty_string() {
        let input = "";
        let expected = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        assert_eq!(hash(input), expected);
    }

    #[test]
    fn test_abcd() {
        assert_eq!(hash("abcd"), "88d4266fd4e6338d13b845fcf289579d209c897823b9217da3e161936f031589");
    }
}
