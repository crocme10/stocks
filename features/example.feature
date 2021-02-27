Feature: Managing currencies

  Scenario: Adding a valid currency
    Given there are no currencies in store
    When I add a currency with code EUR, name Euro, and decimals 2
    Then the response does not contain any error
    And I can find the currency with code EUR
