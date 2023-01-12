# Local Reverse Image Search
![](LRIS_demo_compressed.gif)

**Description**: This program searches a set of directories for instances of some query image. Akaze keypoints are detected in each image using the [akaze crate](https://crates.io/crates/akaze), nearest neighbors are found using the [kdtree crate](https://crates.io/crates/kdtree), "matching" keypoints are determined using Lowe's ratio test [(described in section 7.1 of this paper)](https://www.cs.ubc.ca/~lowe/papers/ijcv04.pdf), and finally images with an "outlier" number of keypoint matches (currently determined by [z-score](https://en.wikipedia.org/wiki/Standard_score)) are reported to the user as overall matches to the query image.

Extracted keypoints and descriptors are cached on disk using the [sled crate](https://crates.io/crates/sled) and recalled in subsequent program executions. 
 
I also view this as a fun playground for Rust stuff, though, so feel free to add any feature you think could be cool!

## Dependencies
All dependencies should be automagically downloaded by Cargo while building.

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


## Open source licensing info
- [LICENSE](LICENSE)


## Credits and references
This project was inspired by work from the University of Minnesota's CSCI 5561 Computer Vision course, taught by Junaed Sattar. The concepts implemented here in Rust are reflected in Python in the course's homework assignments. Thanks to him for teaching the concepts so well!
