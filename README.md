# Steps to run Flex Policy

1. Follow [PDK Prerequisites](https://docs.mulesoft.com/pdk/latest/policies-pdk-prerequisites) on the Mulesoft documentation site, and setup the basic requirements

2. Initialise a new project ```anypoint-cli-v4 pdk policy-project create --name <my-custom-policy>```

3. Copy content of gcl.yaml, lib.rs, Corgo.toml files, and paste these files in your project at respective locations.

4. Compile the project by running ```make build```

5. Publish the policy to mulesoft exchange by running ```make publish```

6. You can now apply this custom policy on your flex gateway api's.

# Steps to run Mule Policy

1. Clone the repo.

2. Run ```git checkout feature/mule_policy```

3. Run ```cd mule-policy/akto```

4. Run ```mvn clean install```

5. Copy the access token from the following curl -  
    ```
        curl --location --request POST 'https://anypoint.mulesoft.com/accounts/login' \
        --header 'Content-Type: application/json' \
        --header 'Accept: application/json' \
        --data-raw '{
            "username":"<anypoint-username>",
            "password":"<anypoint-password>"
        }' | jq -r ".access_token"
    ```

6. Deploy the policy to exchange using following curl (replace org-id and access-token) -  

    ```
        curl --location --request POST 'https://anypoint.mulesoft.com/exchange/api/v2/organizations/         <org-id>/assets/<org-id>/aktopolicy/1.0.0' \
        --header 'Authorization: Bearer <access-token>' \
        --header 'x-sync-publication: true' \
        --form 'files.pom=@"<path-to-cloned-repo>/mule-policy/akto/pom.xml"' \
        --form 'files.mule-policy.jar=@"<path-to-cloned-repo>/mule-policy/akto/target/aktopolicy-1.0.0-mule-policy.jar"' \
        --form 'files.policy-definition.yaml=@"<path-to-cloned-repo>/mule-policy/akto/aktopolicy.yaml"'
    ```
7. You can now apply the policy on your mule runtime api's. The policy will ask for a param (DATA-INGESTION-ENDPOINT), insert the Akto Data Ingestion Service Url here.