# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## v0.4.0 (2023-05-04)

<csr-id-69e8d9f7a45ad95dfe1e3bef00dc53d21df99ad1/>
<csr-id-62393699bc324e6f7ceeac4caad7872116ee7647/>
<csr-id-e5a383c87f846842cf2bd2bed12e6255141ef290/>
<csr-id-85a2b0d50a2300f084cbf996795e7caa10d4155a/>
<csr-id-89c4a9daeed8a6a7c14c817c2ab4df1f9ba2ec65/>
<csr-id-fa8e99fab59704f103e969e0471985279a81bf43/>
<csr-id-20e6c6a68cb4ed429cd24fa72f0de346a4e14978/>
<csr-id-c1c8ff7a7261eec7542e58054614bfb212fb26d0/>
<csr-id-9e68ed34c35eb3a2baac0cb5e46fe355281cd08f/>
<csr-id-b2fed8b9510ae8e0266c220563a4350bf33bfac4/>
<csr-id-aef5978d0126fd163e5d93875b208dc24b7bd9d5/>
<csr-id-5693c8482f2e56b3aa29ef78fefd54b868929b55/>
<csr-id-eeef76a296bdc7b6d962e0933aac2e24137db3f6/>

### Feature

 - <csr-id-69e8d9f7a45ad95dfe1e3bef00dc53d21df99ad1/> Refactored and the MiB logic into its own function, cleaned it up and made it work on all edges properly. Genericized joint probas to be more permissive with args.
 - <csr-id-62393699bc324e6f7ceeac4caad7872116ee7647/> Added a basic demo for the Modifying In Blocks technique. Will save a secondary map, editmap.png, with the top-left quadrant regenerated. Added ability to pass optional args to visualizers.
 - <csr-id-e5a383c87f846842cf2bd2bed12e6255141ef290/> build_unassigned_map_with_size now takes an optional generator function pointer. Ported exploded API to parallel impls. Increased the array cap on smallvecs to allow for blazing-fast generation up to 128x128 maps.
 - <csr-id-85a2b0d50a2300f084cbf996795e7caa10d4155a/> 'Exploded' the build/assign logic into a sane API
   Partially to expose more functionality, partially... in preparation for Modifying In Blocks! Excitemen!
 - <csr-id-89c4a9daeed8a6a7c14c817c2ab4df1f9ba2ec65/> Stopped assigners from needlessly acquiring a write-lock on Finalized nodes
 - <csr-id-fa8e99fab59704f103e969e0471985279a81bf43/> More back-endey optimizations wrt parallelism
 - <csr-id-20e6c6a68cb4ed429cd24fa72f0de346a4e14978/> Replaced ArrayVecs with SmallVecs for adjacency output. Updates examples with new features.
   Rejoice, you can now return any ridiculous amount of neighbors (assuming you enjoy heap allocations).
 - <csr-id-c1c8ff7a7261eec7542e58054614bfb212fb26d0/> Made adjacencies specifiable in the config file as a string with 'adjacency' as the key (case- and whitespace-insensitive). Genericized GeneratorRuleset impl a bit.
 - <csr-id-9e68ed34c35eb3a2baac0cb5e46fe355281cd08f/> Genericized adjacency logic further - instead of having a magic const with unique implementations for cardinal/octile adjacency, adjacency is now handled by an AdjacencyGenerator trait. Fixed up old map2d unit-tests.
   This means new adjacency patterns can be specified by users - for example, an X-shaped 'corner' adjacency, which would previously have conflicted with the 4-neighbor cardinal adjacency.
   NOTE: currently adjacencies for Map2D are hard-capped at 16 elements and WILL error out if you return more, because generics are Painful.
 - <csr-id-b2fed8b9510ae8e0266c220563a4350bf33bfac4/> Genericized number of neighbors returned by adjacent(); not yet exposed in ruleset config though.
 - <csr-id-aef5978d0126fd163e5d93875b208dc24b7bd9d5/> Genericized Position2d to a trait to open it up for expansion into 3D and alternative optimizations.
   This is a fairly rough genericization for now - I know all too well that trying to jump ahead too much will lead to Suffering.

### Chore

 - <csr-id-eeef76a296bdc7b6d962e0933aac2e24137db3f6/> Big Ole Clippy Pass (TM)

### Chore

 - <csr-id-5693c8482f2e56b3aa29ef78fefd54b868929b55/> Big Ole Clippy Pass (TM)

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 12 commits contributed to the release over the course of 17 calendar days.
 - 18 days passed between releases.
 - 12 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Big Ole Clippy Pass (TM) ([`eeef76a`](https://github.com/scrdest/MorkovChain-maptiler/commit/eeef76a296bdc7b6d962e0933aac2e24137db3f6))
    - Refactored and the MiB logic into its own function, cleaned it up and made it work on all edges properly. Genericized joint probas to be more permissive with args. ([`69e8d9f`](https://github.com/scrdest/MorkovChain-maptiler/commit/69e8d9f7a45ad95dfe1e3bef00dc53d21df99ad1))
    - Added a basic demo for the Modifying In Blocks technique. Will save a secondary map, editmap.png, with the top-left quadrant regenerated. Added ability to pass optional args to visualizers. ([`6239369`](https://github.com/scrdest/MorkovChain-maptiler/commit/62393699bc324e6f7ceeac4caad7872116ee7647))
    - Build_unassigned_map_with_size now takes an optional generator function pointer. Ported exploded API to parallel impls. Increased the array cap on smallvecs to allow for blazing-fast generation up to 128x128 maps. ([`e5a383c`](https://github.com/scrdest/MorkovChain-maptiler/commit/e5a383c87f846842cf2bd2bed12e6255141ef290))
    - 'Exploded' the build/assign logic into a sane API ([`85a2b0d`](https://github.com/scrdest/MorkovChain-maptiler/commit/85a2b0d50a2300f084cbf996795e7caa10d4155a))
    - Stopped assigners from needlessly acquiring a write-lock on Finalized nodes ([`89c4a9d`](https://github.com/scrdest/MorkovChain-maptiler/commit/89c4a9daeed8a6a7c14c817c2ab4df1f9ba2ec65))
    - More back-endey optimizations wrt parallelism ([`fa8e99f`](https://github.com/scrdest/MorkovChain-maptiler/commit/fa8e99fab59704f103e969e0471985279a81bf43))
    - Replaced ArrayVecs with SmallVecs for adjacency output. Updates examples with new features. ([`20e6c6a`](https://github.com/scrdest/MorkovChain-maptiler/commit/20e6c6a68cb4ed429cd24fa72f0de346a4e14978))
    - Made adjacencies specifiable in the config file as a string with 'adjacency' as the key (case- and whitespace-insensitive). Genericized GeneratorRuleset impl a bit. ([`c1c8ff7`](https://github.com/scrdest/MorkovChain-maptiler/commit/c1c8ff7a7261eec7542e58054614bfb212fb26d0))
    - Genericized adjacency logic further - instead of having a magic const with unique implementations for cardinal/octile adjacency, adjacency is now handled by an AdjacencyGenerator trait. Fixed up old map2d unit-tests. ([`9e68ed3`](https://github.com/scrdest/MorkovChain-maptiler/commit/9e68ed34c35eb3a2baac0cb5e46fe355281cd08f))
    - Genericized number of neighbors returned by adjacent(); not yet exposed in ruleset config though. ([`b2fed8b`](https://github.com/scrdest/MorkovChain-maptiler/commit/b2fed8b9510ae8e0266c220563a4350bf33bfac4))
    - Genericized Position2d to a trait to open it up for expansion into 3D and alternative optimizations. ([`aef5978`](https://github.com/scrdest/MorkovChain-maptiler/commit/aef5978d0126fd163e5d93875b208dc24b7bd9d5))
</details>

## v0.2.0 (2023-04-15)

<csr-id-4e885810526e735c15235f046cfe17259b88c784/>
<csr-id-dfb7d9130e602e6f08d25ac6c7e86b2fd57a55ad/>
<csr-id-6ae172ecc8b2b1c6284de76e9bf8a2253286696e/>

### Feature

 - <csr-id-4e885810526e735c15235f046cfe17259b88c784/> Officially publishing 0.2.0 - an optimization pass update
 - <csr-id-dfb7d9130e602e6f08d25ac6c7e86b2fd57a55ad/> Another optimization pass - about 25% reduction in RAM usage on 2048x maps.
   Node state (collapsed/uncollapsed) now an enum so we don't store unnecessary data before/after assignment.
   We aggressively prune the enqueued set to remove collapsed tiles (as they will never look it up anymore).
   We switch to using the optimistic size hints when building the map to avoid overcommitting memory.
 - <csr-id-6ae172ecc8b2b1c6284de76e9bf8a2253286696e/> Made map positions type generic. The two main generate*() functions dynamically adapt the storage type used based on map size specified to use the minimum amount of memory.

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release over the course of 12 calendar days.
 - 14 days passed between releases.
 - 3 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - Officially publishing 0.2.0 - an optimization pass update ([`4e88581`](https://github.com/scrdest/MorkovChain-maptiler/commit/4e885810526e735c15235f046cfe17259b88c784))
    - Another optimization pass - about 25% reduction in RAM usage on 2048x maps. ([`dfb7d91`](https://github.com/scrdest/MorkovChain-maptiler/commit/dfb7d9130e602e6f08d25ac6c7e86b2fd57a55ad))
    - Made map positions type generic. The two main generate*() functions dynamically adapt the storage type used based on map size specified to use the minimum amount of memory. ([`6ae172e`](https://github.com/scrdest/MorkovChain-maptiler/commit/6ae172ecc8b2b1c6284de76e9bf8a2253286696e))
</details>

## v0.1.0 (2023-04-01)

<csr-id-706b2e0442678ab68ec21cd85aa55380cc110a52/>
<csr-id-1f48f6debfbe67746d091532a058e16ffdcf47af/>

### Chore

 - <csr-id-706b2e0442678ab68ec21cd85aa55380cc110a52/> More documentation!
 - <csr-id-1f48f6debfbe67746d091532a058e16ffdcf47af/> Big Bang first pass

### Commit Statistics

<csr-read-only-do-not-edit/>

 - 3 commits contributed to the release.
 - 2 commits were understood as [conventional](https://www.conventionalcommits.org).
 - 0 issues like '(#ID)' were seen in commit messages

### Commit Details

<csr-read-only-do-not-edit/>

<details><summary>view details</summary>

 * **Uncategorized**
    - More documentation! ([`706b2e0`](https://github.com/scrdest/MorkovChain-maptiler/commit/706b2e0442678ab68ec21cd85aa55380cc110a52))
    - Big Bang first pass ([`1f48f6d`](https://github.com/scrdest/MorkovChain-maptiler/commit/1f48f6debfbe67746d091532a058e16ffdcf47af))
    - Initial commit ([`0ed6362`](https://github.com/scrdest/MorkovChain-maptiler/commit/0ed636258926731f511a8ce8859562992ae64781))
</details>

