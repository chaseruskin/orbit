//! Overdetermined system solver

/// Given a partially defined `target`, reduce the `space` for solutions that match
/// the `target` as much as possible.
pub fn reduce<T>(mut space: Vec<Option<Vec<T>>>, target: &[Option<T>]) -> Vec<Option<Vec<T>>> 
    where T: std::cmp::PartialEq {
    // at each level provide filter until target provides no more information
    let mut target_iter = target.iter().enumerate();
    while let Some((i, Some(t))) = target_iter.next() {
        space
            .iter_mut()
            .for_each(|f| {
                if f.is_none() || f.as_ref().unwrap().get(i).is_none() || f.as_ref().unwrap().get(i).unwrap() != t {
                    f.take()
                } else {
                    None
                };
        });
    }
    space
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn one_soln() {
        let space = vec![
            Some(vec![10, 0, 9]),
            Some(vec![4, 1, 2]),
            Some(vec![0, 1, 2]),
        ];
        assert_eq!(reduce(space, &vec![Some(4), None, None]), vec![
            None, 
            Some(vec![4, 1, 2]), 
            None,
        ]);

        let space = vec![
            Some(vec![10, 0, 9]),
            Some(vec![4, 1, 2]),
            Some(vec![4, 2, 5]),
        ];
        assert_eq!(reduce(space, &vec![Some(4), Some(2), None]), vec![
            None, 
            None, 
            Some(vec![4, 2, 5]),
        ]);
    }

    #[test]
    fn mult_soln() {
        let space = vec![
            Some(vec![10, 0, 9]),
            Some(vec![4, 1, 2]),
            Some(vec![0, 1, 2]),
            Some(vec![4, 9, 5]),
        ];
        assert_eq!(reduce(space, &vec![Some(4), None, None]), vec![
            None, 
            Some(vec![4, 1, 2]), 
            None,
            Some(vec![4, 9, 5]),
        ]);
    }

    use std::str::FromStr;
    use crate::core::pkgid;

    #[test]
    fn reduce_pkgid() {
        // one solution
        let space = vec![
            Some(pkgid::PkgId::from_str("ks-tech.rary1.ip1").unwrap().into_full_vec().unwrap()),
            Some(pkgid::PkgId::from_str("ks-tech.rary1.ip2").unwrap().into_full_vec().unwrap()),
            Some(pkgid::PkgId::from_str("ks-tech.rary1.ip3").unwrap().into_full_vec().unwrap()),
            Some(pkgid::PkgId::from_str("ks-tech.rary2.ip1").unwrap().into_full_vec().unwrap()),
            Some(pkgid::PkgId::from_str("ks-tech.rary2.ip2").unwrap().into_full_vec().unwrap()),
        ];
        let target = pkgid::PkgId::from_str("ip3").unwrap().into_vec();
        assert_eq!(reduce(space, target.as_slice()), vec![
            None,
            None, 
            Some(vec![pkgid::PkgPart::from_str("ip3").unwrap(), pkgid::PkgPart::from_str("rary1").unwrap(), pkgid::PkgPart::from_str("ks-tech").unwrap()]), 
            None, 
            None
        ]);

        // one solution (comparing as `PkgPart`)
        let space = vec![
            Some(pkgid::PkgId::from_str("ks-tech.rary1.ip1").unwrap().into_full_vec().unwrap()),
            Some(pkgid::PkgId::from_str("ks-tech.rary1.ip2").unwrap().into_full_vec().unwrap()),
            Some(pkgid::PkgId::from_str("ks-tech.rary1.ip3").unwrap().into_full_vec().unwrap()),
            Some(pkgid::PkgId::from_str("ks-tech.rary2.ip1").unwrap().into_full_vec().unwrap()),
            Some(pkgid::PkgId::from_str("ks-tech.rary2.ip2").unwrap().into_full_vec().unwrap()),
        ];
        let target = pkgid::PkgId::from_str("IP3").unwrap().into_vec();
        assert_eq!(reduce(space, target.as_slice()), vec![
            None,
            None, 
            Some(vec![pkgid::PkgPart::from_str("ip3").unwrap(), pkgid::PkgPart::from_str("rary1").unwrap(), pkgid::PkgPart::from_str("ks-tech").unwrap()]), 
            None, 
            None
        ]);

        // no solution
        let space = vec![
            Some(pkgid::PkgId::from_str("ks-tech.rary1.ip1").unwrap().into_full_vec().unwrap()),
            Some(pkgid::PkgId::from_str("ks-tech.rary1.ip2").unwrap().into_full_vec().unwrap()),
            Some(pkgid::PkgId::from_str("ks-tech.rary1.ip3").unwrap().into_full_vec().unwrap()),
            Some(pkgid::PkgId::from_str("ks-tech.rary2.ip1").unwrap().into_full_vec().unwrap()),
            Some(pkgid::PkgId::from_str("ks-tech.rary2.ip2").unwrap().into_full_vec().unwrap()),
        ];
        let target = pkgid::PkgId::from_str("ip3.unknown").unwrap().into_vec();
        assert_eq!(reduce(space, target.as_slice()), vec![
            None,
            None, 
            None,
            None, 
            None
        ]);

        // multiple solutions
        let space = vec![
            Some(pkgid::PkgId::from_str("ks-tech.rary1.ip1").unwrap().into_full_vec().unwrap()),
            Some(pkgid::PkgId::from_str("ks-tech.rary1.ip2").unwrap().into_full_vec().unwrap()),
            Some(pkgid::PkgId::from_str("ks-tech.rary1.ip3").unwrap().into_full_vec().unwrap()),
            Some(pkgid::PkgId::from_str("ks-tech.rary2.ip1").unwrap().into_full_vec().unwrap()),
            Some(pkgid::PkgId::from_str("ks-tech.rary2.ip2").unwrap().into_full_vec().unwrap()),
        ];
        let target = pkgid::PkgId::from_str("ip2").unwrap().into_vec();
        assert_eq!(reduce(space, target.as_slice()), vec![
            None,
            Some(pkgid::PkgId::from_str("ks-tech.rary1.ip2").unwrap().into_full_vec().unwrap()), 
            None,
            None, 
            Some(pkgid::PkgId::from_str("ks-tech.rary2.ip2").unwrap().into_full_vec().unwrap())
        ]);
    }
}