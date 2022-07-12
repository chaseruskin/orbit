use crate::util::overdetsys;
use crate::core::pkgid::PkgPart;
use crate::util::anyerror::AnyError;
use super::pkgid::PkgId;

/// Given a partial/full ip specification `ip_spec`, sift through the manifests
/// for a possible determined unique solution.
/// 
/// Note: Currently clones each id, possibly look for faster implemtenation avoiding clone.
pub fn find_ip(ip_spec: &PkgId, universe: Vec<&PkgId>) -> Result<PkgId, AnyError> {
    // try to find ip name
    let space: Vec<Vec<PkgPart>> = universe.into_iter().map(|f| { f.into_full_vec().unwrap() }).collect();
    let result = match overdetsys::solve(space, ip_spec.iter()) {
        Ok(r) => r,
        Err(e) => match e {
            overdetsys::OverDetSysError::NoSolution => Err(AnyError(format!("no ip as '{}' exists", ip_spec)))?,
            overdetsys::OverDetSysError::Ambiguous(set) => {
                // assemble error message
                let mut set = set.into_iter().map(|f| PkgId::from_vec(f) );
                let mut content = String::new();
                while let Some(s) = set.next() {
                    content.push_str(&format!("    {}\n", s.to_string()));
                }
                Err(AnyError(format!("ambiguous ip '{}' yields multiple solutions:\n{}", ip_spec, content)))?
            }
        }
    };
    Ok(PkgId::from_vec(result))
}