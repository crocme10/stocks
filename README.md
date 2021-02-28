# Stocks GraphQL Service

A GraphQL interface to expose the Stocks Service. The Stocks Service is a toy implementation

## Summary

  - [Getting Started](#getting-started)
  - [Runing the tests](#running-the-tests)
  - [Deployment](#deployment)
  - [Built With](#built-with)
  - [Contributing](#contributing)
  - [Versioning](#versioning)
  - [Authors](#authors)
  - [License](#license)
  - [Acknowledgments](#acknowledgments)

## Getting Started

These instructions will get you a copy of the project up and running on
your local machine for development and testing purposes. See deployment
for notes on how to deploy the project on a live system.

### Prerequisites

What things you need to install the software and how to install them

    Give examples

### Installing

A step by step series of examples that tell you how to get a development
env running

Say what the step will be

    Give the example

And repeat

    until finished

End with an example of getting some data out of the system or using it
for a little demo

### Scratching the interface.

This GraphQL interface comes with a playground, which is a GraphQL IDE. This web application allows you to discover the GraphQL schema, and run queries,
mutations, and subscriptions. It's a very convenient way to test your interface.

The playground is available at `..../playground`.

Another useful tool to test the interface is `curl` (or `wget`).

```
curl -X POST "http://localhost:8080/graphql" -H 'Content-Type: application/json' --data-binary @payload.json
```

where the `payload.json`:

```json
{ "query": "query listCurrencies { listCurrencies { code, name, decimals } }" }
```

## Running the tests

Tests include both unit tests and some integration tests. Some of these tests require the backend database,
which is available as a docker image.

### Start database backend

Obviously you should make sure you're not targetting some kind of production database...
We force the user to specify to environment variables for all the tests that actually connect
to a database:

- **RUN_MODE** should have the value **TESTING**
- **DATABASE_TEST_URL**

The database backend is configured with `config/test.env`, which is a dotenv format:

```
DB_REPO=localhost:5000
DB_IMAGE=stocks/db
DB_TAG=0.2.0
DB_PORT=5431
```

Having set proper values for this file, you can now start the database as a daemon:

```shell
docker-compose -f docker-compose-test.yml --env-file config/test.env up -d
```

### Execute tests

As mentioned previously, you need to specify both `RUN_MODE=testing`, and `DATABASE_TEST_URL` environment variables:

```
RUN_MODE=testing DATABASE_TEST_URL=postgres://bob:secret@localhost:5431/stocks cargo test --release
```

You can target just unit tests by using `--libs`, and integration tests using `--tests`

### Integration tests

Integration tests are written using **cucumber** and **gherkin**. Tests are specified in the `features` directory, and the code for implementing these tests
are in the `tests` directory. These tests don't start an actual server, but they execute requests against the GraphQL schema, and analyze the results.

## Deployment

Add additional notes about how to deploy this on a live system

## Built With

  - [Contributor Covenant](https://www.contributor-covenant.org/) - Used
    for the Code of Conduct
  - [Creative Commons](https://creativecommons.org/) - Used to choose
    the license

## Contributing

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code
of conduct, and the process for submitting pull requests to us.

## Versioning

We use [SemVer](http://semver.org/) for versioning. For the versions
available, see the [tags on this
repository](https://github.com/PurpleBooth/a-good-readme-template/tags).

## Authors

  - **Billie Thompson** - *Provided README Template* -
    [PurpleBooth](https://github.com/PurpleBooth)

See also the list of
[contributors](https://github.com/PurpleBooth/a-good-readme-template/contributors)
who participated in this project.

## License

This project is licensed under the [CC0 1.0 Universal](LICENSE.md)
Creative Commons License - see the [LICENSE.md](LICENSE.md) file for
details

## Acknowledgments

  - Hat tip to anyone whose code was used
  - Inspiration
  - etc

