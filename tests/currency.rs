use cucumber::async_trait;
use sqlx::postgres::PgPoolOptions;
use std::convert::Infallible;
use std::time::Duration;

use stocks::api::{gql, imp};
use stocks::utils;

pub struct MyWorld {
    schema: gql::StocksSchema,
    response: async_graphql::Response,
}

#[async_trait(?Send)]
impl cucumber::World for MyWorld {
    // TODO Can we have another type?
    type Error = Infallible;

    async fn new() -> Result<Self, Infallible> {
        // TODO Can we make it fail cleanly and informatively
        let url = utils::get_database_url();
        let pool = PgPoolOptions::new()
            .connect_timeout(Duration::new(2, 0))
            .connect(&url)
            .await
            .expect("Database connection");
        let service = imp::StockServiceImpl { pool };
        Ok(Self {
            response: async_graphql::Response::new(()),
            schema: gql::schema(Box::new(service)),
        })
    }
}

mod example_steps {
    use async_graphql::{value, Variables};
    use cucumber::{t, Steps};

    pub fn steps() -> Steps<crate::MyWorld> {
        let mut builder: Steps<crate::MyWorld> = Steps::new();

        builder
            .given_async(
                "there are no currencies in store",
                t!(|mut world, _step| {
                    // TODO Call delete_all_currencies.
                    world
                }),
            )
            .when_regex_async(
                r"I add a currency with code ([A-Z]{3}), name ([a-zA-Z]+), and decimals (\d+)",
                t!(|mut world, matches, _step| {
                    let code = matches[1].to_owned();
                    let name = matches[2].to_owned();
                    let decimals = matches[3].to_owned().parse::<i32>().unwrap();
                    let request = async_graphql::Request::new(
                        r#"
                        mutation addCurrency($currency: CurrencyInput!) {
                            addCurrency(currency: $currency) {
                                code, name, decimals
                            }
                        }
                      "#,
                    )
                    .variables(Variables::from_value(value!({
                        "currency": {
                            "code": code,
                            "name": name,
                            "decimals": decimals
                        }
                    })));
                    world.response = world.schema.execute(request).await;
                    world
                }),
            )
            .then(
                r"the response does not contain any error",
                |world, _step| {
                    if world.response.is_err() {
                        println!("resp: {:?}", world.response);
                        println!(); // TODO Not sure why I need this...
                    }
                    assert!(world.response.is_ok());
                    world
                },
            )
            .then_regex_async(
                r"^I can find the currency with code ([A-Z]{3})$",
                t!(|world, matches, _step| {
                    let code = matches[1].to_owned();
                    let request = async_graphql::Request::new(
                        r#"
                        query findCurrency($code: String!) {
                            findCurrency(code: $code) {
                                code, name, decimals
                            }
                        }
                      "#,
                    )
                    .variables(Variables::from_value(value!({
                        "code": code,
                    })));
                    let response = world.schema.execute(request).await;
                    if response.is_err() {
                        for err in response.errors {
                            println!("{}", err);
                        }
                        println!("");
                    } else {
                        assert_eq!(
                            response.data,
                            value!({
                                    "findCurrency": {
                                        "code": "EUR",
                                        "name": "Euro",
                                        "decimals": 2
                                    }
                            })
                        );
                        // println!("data: {}", response.data);
                        // println!("");
                    }
                    world
                }),
            );

        builder
    }
}

#[tokio::main]
async fn main() {
    // Do any setup you need to do before running the Cucumber runner.
    // e.g. setup_some_db_thing()?;

    cucumber::Cucumber::<MyWorld>::new()
        // Specifies where our feature files exist
        .features(&["./features"])
        // Adds the implementation of our steps to the runner
        .steps(example_steps::steps())
        // Parses the command line arguments if passed
        .cli()
        // Runs the Cucumber tests and then exists
        .run_and_exit()
        .await
}
