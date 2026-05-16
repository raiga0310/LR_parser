#[derive(Debug, PartialEq, Eq)]
pub enum Validation<E, T> {
    Valid(T),
    Invalid(Vec<E>),
}

impl<E, T> Validation<E, T> {
    pub fn valid(value: T) -> Self {
        Validation::Valid(value)
    }

    pub fn invalid(error: E) -> Self {
        Validation::Invalid(vec![error])
    }

    pub fn from_result(r: Result<T, E>) -> Self {
        match r {
            Ok(v)  => Validation::Valid(v),
            Err(e) => Validation::Invalid(vec![e]),
        }
    }

    #[allow(dead_code)]
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Validation<E, U> {
        match self {
            Validation::Valid(v)       => Validation::Valid(f(v)),
            Validation::Invalid(errs)  => Validation::Invalid(errs),
        }
    }

    /// 互いに独立な2つの検証結果を合成する。
    /// 両方成功 → f(t, u) を Valid で返す。
    /// 片方以上が失敗 → 両側のエラーをすべて収集して Invalid で返す。
    pub fn map2<U, V, F>(self, other: Validation<E, U>, f: F) -> Validation<E, V>
    where
        F: FnOnce(T, U) -> V,
    {
        match (self, other) {
            (Validation::Valid(t), Validation::Valid(u)) => Validation::Valid(f(t, u)),
            (Validation::Invalid(e), Validation::Valid(_)) => Validation::Invalid(e),
            (Validation::Valid(_), Validation::Invalid(e)) => Validation::Invalid(e),
            (Validation::Invalid(mut e1), Validation::Invalid(mut e2)) => {
                e1.append(&mut e2);
                Validation::Invalid(e1)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map2_both_valid_returns_combined() {
        let a: Validation<&str, i32> = Validation::valid(1);
        let b: Validation<&str, i32> = Validation::valid(2);
        assert_eq!(a.map2(b, |x, y| x + y), Validation::Valid(3));
    }

    #[test]
    fn map2_left_invalid_returns_left_errors() {
        let a: Validation<&str, i32> = Validation::invalid("err_a");
        let b: Validation<&str, i32> = Validation::valid(2);
        assert_eq!(a.map2(b, |x, y| x + y), Validation::Invalid(vec!["err_a"]));
    }

    #[test]
    fn map2_right_invalid_returns_right_errors() {
        let a: Validation<&str, i32> = Validation::valid(1);
        let b: Validation<&str, i32> = Validation::invalid("err_b");
        assert_eq!(a.map2(b, |x, y| x + y), Validation::Invalid(vec!["err_b"]));
    }

    #[test]
    fn map2_both_invalid_accumulates_all_errors_in_order() {
        let a: Validation<&str, i32> = Validation::invalid("err_a");
        let b: Validation<&str, i32> = Validation::invalid("err_b");
        assert_eq!(
            a.map2(b, |x, y| x + y),
            Validation::Invalid(vec!["err_a", "err_b"])
        );
    }
}
