# Tethers Deprecation Policy

Status: living policy  
Updated: 2026-09-14

For Tethers 1.x:

- Stable public behaviour is not removed in a minor release without a
  documented transition, except where security or correctness makes continued
  behaviour unsafe.
- Machine contracts expose version changes explicitly.
- Deprecated interfaces identify their replacement when one exists.
- Internal implementation structures receive no compatibility promise.
- Experimental interfaces are visibly marked experimental before users could
  reasonably mistake them for stable.
- Security fixes may override ordinary deprecation timing when necessary.
- No permanent promise is created accidentally through an undocumented
  experimental feature.

Deprecation notices should state the affected versioned surface, the reason,
the replacement or migration where available, and the release boundary at
which removal could occur. This policy makes no unrealistic calendar-based
guarantee.

