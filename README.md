# Local Reverse Image Search

![](LRIS_demo_compressed.gif)

**Description**: This program searches a set of directories for instances of some query image. Akaze keypoints are detected in each image using the AKAZE CRATE (ADD LINK), nearest neighbors are found using the KDTree crate (ADD LINK), matches are calculated using the ratio test (ADD LINK), and finally images with an "outlier" number of matches are reported to the user as matches.

I also view this as a fun playground for Rust stuff, though, so feel free to add any feature you think could be cool!

TODO:
- Finish custom serialization/deserialization
- Finish caching implementation

## Dependencies

No dependencies outside of those specified in the Cargo.toml file should be needed.

## Installation

1. Install Rust [(guide here)](https://www.rust-lang.org/tools/install)
2. Clone this repository ```git clone https://github.com/benhamlen/local-reverse-image-search.git```

## Configuration
```config.toml``` is the main configuration document for this program.

The most important configuration is the search directory paths.

## Usage
1. Run the program with ```cargo run --release```
2. Select a query image

## How to test the software

No tests for now, perhaps will add some in the future. Was thinking about characterizing the program's performance by randomly selecting many query images and seeing what images it has trouble with, what images it detects well, etc.

## Known issues

None for now, certainly some exist.

If you have questions, concerns, bug reports, etc, please file an issue in this repository's Issue Tracker.

## Getting involved

This section should detail why people should get involved and describe key areas you are
currently focusing on; e.g., trying to get feedback on features, fixing certain bugs, building
important pieces, etc.

General instructions on _how_ to contribute should be stated with a link to [CONTRIBUTING](CONTRIBUTING.md).


----

## Open source licensing info
1. [TERMS](TERMS.md)
2. [LICENSE](LICENSE)
3. [CFPB Source Code Policy](https://github.com/cfpb/source-code-policy/)

----

## Credits and references

1. Projects that inspired you
2. Related projects
3. Books, papers, talks, or other sources that have meaningful impact or influence on this project
