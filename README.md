# About
This tool is for processing coverage data on minified javascript.

Specifically, it can read chrome coverage data (from puppeteer), and for any files that reference a source map, calculate the line coverage of the contributing source files.

It requires a configuration file to understand where the referenced files are, and how to traverse and interpret the source maps.

## Requirements

### Building
This tool is built using Rust. 1.28 and greater should work for building it. I think it can be built on low of a version as 1.26.

### Inputs
This tool processes chrome coverage data as generated from puppeteer. It requires you to have the source files involved for the traversing, specifically:
 * The source files
 * The minified sources
 * The source maps referenced from the source files
 
### Outputs
This tool outputs coverage data in the [sonarqube coverage format](https://docs.sonarqube.org/display/SONAR/Generic+Test+Data). More output formats may be added in the future

## Using the tool 

### Generating the input coverage data
The type of data this tool takes in is chrome coverage data (from puppeteer). It's generated using the [startJSCoverage](https://github.com/GoogleChrome/puppeteer/blob/master/docs/api.md#coveragestartjscoverageoptions) on the page object. Here's an example to output the coverage data to the `some_action.json` file in the `coverage` folder (this code assumes that that folder already exists).

```js
// Start the coverage
page.coverage.startJSCoverage({ resetOnNavigation: false }) ;

// Do stuff with the page, like visiting your website
// thus generating coverage data

// End the coverage
let jsCoverage = await page.coverage.stopJSCoverage();
var json = JSON.stringify(jsCoverage);
var file_name = "coverage/some_action.json"
fs.writeFile(file_name, json, "utf8", (error: any) => {
    /* handle error */
});
```

### Configuring 

To run atlas on the coverage data, you'll need a json configuration file for it. unless otherwise indicated, all urls and paths require a trailing slash. Here are the keys you'll need.

**public_url_base**: This is the full url prefix of where you're serving files from. It's what the tool uses to filter out the initial coverage data. If you had all your static js code at `http://localhost/assets/js/` for example, that's what you'd use here. 

**dist_path**: This is the absolute path your minified files on disk.

**dist_coverage_url**: This is the url prefix used in sourcemaps to reference source files

**dist_coverage_path**: This is used to locate source files referenced in source maps, by replacing the `dist_coverage_url` with the `dist_coverage_path`. Needs to be an absolute path.

**reify_against_lcov**: This is optional. It's an abosolute path to lcov data from unit tests against the source files. This is used to strip out line hits from object instantiation.

**sources**: This is an object with three fields:

**sources.base**: The base url to perform filtering at

**sources.dirs**: An array of globs, relative to the `sources.base` value. Anything source file referenced in a source map is excluded from processing if it doesn't match one of these globs.

**sources.excludes**: An array of globs, relative to the `sources.base` value. It's applied after the dirs to exclude other specific items.

#### Example
```json
{
    "public_url_base": "http://localhost/assets/js/",
    "dist_path": "/Users/sam/projects/example/dist",
    "dist_coverage_path": "/Users/sam/projects/example/builder",
    "dist_coverage_url": "webpack:///",
    "sources": {
        "base": "/Users/sam/projects/example",
        "dirs": [
            "src/**",
        ],
        "excludes": [
            "**/*.spec.js",
            "**/*.auto-generated-nav-*.js"
        ]
    },
    "reify_against_lcov": "/Users/sam/projects/example/test-results/lcov/coverage/lcov.info"
}
```

### Running

With coverag data and a config file to find the sources, you can run the tool like so:

`atlas-coverage --output /abs/path/output-folder/output.xml --config /abs/path/to/config.json /abs/path/to/coverage/json/data`

This assumes the following:
1) You have the atlas-coverage binary in your path
2) `/abs/path/output-folder` exists
3) You have a valid config at `/abs/path/to/config.json`
4) The `/abs/path/to/coverage/json/data` exists and has at least one puppeteer json coverage file in it

## Caveats

### Output Format
Right now we only output in the sonarqube format. It was definitely the simplest thing to do, and other output files are welcome.

### Coverage Accuracy
Coverage data is still a little iffy. We only output line coverage, no condition/branch coverage. And without lcov data to reify against, the line coverage data reports more executable lines of code than actually exist.

Currently we using mapping data to map from the minified file back to the source file. If we find mapping data for a particular line, we count that as an executable line. That's different from other tools that generate lcov data. One such example would be for object instatiation. For example this code:

```js
let a = "hello world";

let b = {
  greeting: a,
  value: 11
}
```

Without lcov reification, this tool would mark the instation of `b` as being multiple executable lines of code.

This is a symptom of a greater issue at this point, which is that we don't actually parse the javascript (source code or minified). If you know of a good integratable javascript parser, please drop me a line.

## Roadmap

Better error handling, configurable output formats and actually parsing the source files to generate condition/branch coverage are the major roadmap items.

## Contributing

If you'd like to contribute you can open an issue or a pull request with proposed improvements.
