

# INTEGRATOR REST API'S

Version: 2.0.1

# Available endpoints

| Endpoint                                | Method | Action |
| --------------------------------------- | ------ | ------------------------------------------------------------ |
| /service/authenticate                   | POST   | Obtain an authorization token (needed as Bearer authentication for all other requests) |
| /eas/documents                          | POST   | Upload a document with metadata |
| /eas/documents/{ticket}                 | GET    | Download a document or download specific file from an archived object |
| /eas/documents/{ticket}/contentList     | GET    | Get content list of an archived object |
| /eas/documents/{ticket}/metadata        | GET    | Get documents metadata |
| /eas/documents/{ticket}/metadata        | PATCH  | Update documents metadata |
| /eas/documents/{ticket}                 | DELETE | Delete a document |
| /eas/documents                          | GET    | Get  matching documents |



# Authenticate

POST /service/authenticate

<u>Request body (application/json):</u>

```json
{
  "appId": "your application id",
  "appToken": "your application token",
  "accountName": "reference to the user in the name of and on behalf of you are executing the request"
}
```

<u>Responses:</u>

- Code 200: OK


  Response body (application/json):

  ```json
  {
    "token": "your authorization token"
  }
  ```

- Code 400: Bad request

  Response body (application/json):

  ```json
  {
    "Message":"Detailed error message"
  }
  ```

- Code 401: Unauthorized



<u>curl example:</u>

```sh
curl --location --request POST 'https://.../service/authenticate' \
--header 'Accept: application/json' \
--header 'Content-Type: application/json' \
--data-raw '{"appId":"your application id","appToken":"your application token","accountName":"reference to the user in the name of and on behalf of you are executing the request"}'
```

<u>curl response body:</u>

```json
{
  "token": "your authorization token"
}
```



# Upload a document

POST /eas/documents

<u>Request headers:</u>

- Authorization

<u>Request body (multipart/form-data):</u>

- fingerprint

- fingerprintAlgorithm (NONE, MD5, SHA-1, SHA-256, SHA-512)

- document (binary)

- metadata (list of name value pairs)

  

<u>Responses:</u>

- Code 200: OK


  Response body (application/json):

  ```json
  {
    "ticket": "unique reference towards the uploaded document"
  }
  ```

- Code 400: Bad request

  Response body (application/json):

  ```json
  {
    "Message":"Detailed error message"
  }
  ```

- Code 401: Unauthorized

  

<u>curl example:</u>

```shell
curl --location --request POST 'https://.../eas/documents' \
--header 'Authorization: Bearer your authorization token' \
--form 'document=@"CRA.pdf"' \
--form 'fingerprint=""' \
--form 'fingerprintAlgorithm="none"' \
--form 'metadata="[{name: \"ClientId\", value: \"1\"}, {name: \"CustomerId\", value: \"2\"}]"'
```

<u>curl response body:</u>

```json
{
  "ticket": "A_22930F1B08684C418212613C6EFEEBF3_1"
}
```



# Download a document

GET /eas/documents/{ticket}

<u>Request headers:</u>

- Authorization

<u>Request parameters:</u>

- FileName (optional): 
  <br>- used to get a specific file from the archived object
  <br>- The filename is case sensitive and can be retrieved by the contentList api call

<u>Responses:</u>

- Code 200: OK


  Response body (application/json):

  ```json
  {
    "mimeType": "documents MIME-type (eq. application.pdf, text/plain, ...) ",
    "base64Document": "document base64 encoded "
  }
  ```

- Code 400: Bad request

  Response body (application/json):

  ```json
  {
    "Message":"Detailed error message"
  }
  ```

- Code 401: Unauthorized

- Code 404: Not found

  

<u>curl example:</u>

```sh
curl --location --request GET 'https://.../eas/documents/A_FCBDAA9949D04B2980041F98E35AD05C_1' \
--header 'Accept: application/json' \
--header 'Authorization: Bearer your authorization token'
```

<u>curl example with fileName as parameter:</u>

```sh
curl --location --request GET 'https://.../eas/documents/A_FCBDAA9949D04B2980041F98E35AD05C_1?fileName=myFile1.pdf' \
--header 'Accept: application/json' \
--header 'Authorization: Bearer your authorization token'
```


<u>curl response body:</u>

```json
{
  "mimeType":"application/pdf",
  "base64Document":"JVBERi0xLjQKMSAwIG9iago8PAo...=="
}
```



# Get content of an archived object

GET /eas/documents/{ticket}/contentList

<u>Request headers:</u>

- Authorization

<u>Responses:</u>

- Code 200: OK


  Response body (application/json):

  ```json
  [
    "", ...
  ]
  ```

- Code 400: Bad request

  Response body (application/json):

  ```json
  {
    "Message":"Detailed error message"
  }
  ```

- Code 401: Unauthorized

- Code 404: Not found

  

<u>curl example:</u>

```sh
curl --location --request GET 'https://.../eas/documents/A_FCBDAA9949D04B2980041F98E35AD05C_1/contentList' \
--header 'Accept: application/json' \
--header 'Authorization: Bearer your authorization token'
```

<u>curl response body:</u>

```json
[
  "myFile1.pdf",
  "myFile2.pdf"
]
```



# Get documents metadata

GET /eas/documents/{ticket}/metadata

<u>Request headers:</u>

- Authorization

<u>Responses:</u>

- Code 200: OK


  Response body (application/json):

  ```json
  {
    "mimeType": "documents MIME-type (eq. application.pdf, text/plain, ...) ",
    "base64Document": "document base64 encoded "
  }
  ```

- Code 400: Bad request

  Response body (application/json):

  ```json
  {
    "Message":"Detailed error message"
  }
  ```

- Code 401: Unauthorized

- Code 404: Not found

  

<u>curl example:</u>

```sh
curl --location --request GET 'https://.../eas/documents/A_FCBDAA9949D04B2980041F98E35AD05C_1/metadata' \
--header 'Accept: application/json' \
--header 'Content-Type: application/json' \
--header 'Authorization: Bearer your authorization token'
```

<u>curl response body:</u>

```json
{
  "metadata": [
    {
      "name": "Import Date",
      "value": "11/03/2021"
    },
    {
      "name": "ClientId",
      "value": "123456789"
    },
    {
      "name": "CustomerId",
      "value": "AZER456"
    },
    {
      "name": "Documenttype",
      "value": "Invoice"
    }
  ]
}
```



# Update documents metadata

PATCH /eas/documents/{ticket}/metadata

<u>Request headers:</u>

- Authorization

<u>Responses:</u>

- Code 200: OK

- Code 400: Bad request

  Response body (application/json):

  ```json
  {
    "Message":"Detailed error message"
  }
  ```

- Code 401: Unauthorized

- Code 404: Not found

  

<u>curl example:</u>

```sh
curl --location --request PATCH 'https://.../eas/documents/A_FCBDAA9949D04B2980041F98E35AD05C_1/metadata' \
--header 'Content-Type: application/json' \
--header 'Authorization: Bearer your authorization token' \
--data-raw '{"metaData":[{"name":"ClientId","value": "2"}]}'
```



# Delete document

DELETE /eas/documents/{ticket}

<u>Request headers:</u>

- Authorization

<u>Request parameters:</u>

- Motivation (required)

<u>Responses:</u>

- Code 200: OK

- Code 400: Bad request

  Response body (application/json):

  ```json
  {
    "Message":"Detailed error message"
  }
  ```

- Code 401: Unauthorized

- Code 404: Not found

  

<u>curl example:</u>

```sh
curl --location --request DELETE 'https://.../eas/documents/A_21967ED428F94E679296D48B6E198285_1?motivation=ReasonWhyWeDelete' \
--header 'Authorization: Bearer your authorization token'
```



# Get matching documents

GET /eas/documents/

<u>Request headers:</u>

- Authorization

<u>Request parameters:</u>

- pageNumber

- pageSize

- filter

- fields

- sortBy

  

fields = comma separated list of metadata filenames

sortBy = metadata fieldname asc |desc

filter = comma separated list of **field[operator]value** format

Supported operators:

| Operator | Meaning                  |
| -------- | ------------------------ |
| sw       | starts with              |
| ew       | ends with                |
| eq       | equal to                 |
| ne       | not equal to             |
| lt       | less than                |
| lte      | less than or equal to    |
| gt       | greater than             |
| gte      | greater than or equal to |



<u>Responses:</u>

- Code 200: OK


  Response body (application/json):

  ```json
  {
    "data": [],
    "pageSize": 0,
    "pageNumber": 0,
    "totalPages": 0,
    "totalRecords": 0
  }
  ```

- Code 400: Bad request

  Response body (application/json):

  ```json
  {
    "Message":"Detailed error message"
  }
  ```

- Code 401: Unauthorized



<u>curl example:</u>

```sh
curl --location --request GET 'https://.../eas/documents?fields=ClientId,CustomerId,Documenttype&filter=ClientId[eq]*&pageNumber=1&pageSize=20&sortBy=' \
--header 'Accept: application/json' \
--header 'Authorization: Bearer your authorization token'
```

<u>curl response body:</u>

```json
{
  "data": [
    {
      "ticket": "A_C64C95787F264E70AF0783DE47A5EA32_1",
      "ClientId": "1",
      "CustomerId": "1",
      "Documenttype": "Invoice"
    },
    {
      "ticket": "A_FCBDAA9949D04B2980041F98E35AD05C_1",
      "ClientId": "2",
      "CustomerId": "2",
      "Documenttype": "Payslip"
    }
  ],
  "pageSize": 20,
  "pageNumber": 1,
  "totalPages": 1,
  "totalRecords": 2
}
```

# Configuration file
## Description: 
* Located in the root directory of the api: **appSettingsEncrypted.json**
* Can be decrypted and edited with the SettingsFileEditor application
* Contains a list of 'profiles' which each point to a specific EAS domain and document type
## Content:
```json
{
  "profiles": [
    {
      "timeOutInMinutes": 20,
      "integratorsGroupName": "Integrators",
      "appId": "f33c398c-0f77-4351-9f92-1e20fa3fd2f8",
      "appToken": "e1320735-e174-4150-9edb-b5daf85be6d1",
      "domain": "DOMAIN_NAME",
      "documentType": "Document type name",
      "technicalUserName": "user",
      "technicalUserPassword": "password",
      "active": true,
      "async": true,
      "asyncEventHandler": "EventHandlerName",
      "asyncEventHandlerParams": {
        "A4sbInputDirectory": "InputDirectory"
      }
    }
  ]
}
```
## Parameters:
* **timeOutInMinutes** (int): defines the time in minutes that the token is valid
* **integratorsGroupName** (string): EAS Role to user for the accounts, need crud on deposits, query, crud on metadata rights
* **appId** (string): Unique application id (usually we use the same per domain)
* **appToken** (string): Unique application token
* **domain** (string): EAS domain name
* **documentType** (string): EAS document type name
* **technicalUserName** (string): EAS technical account used to create the account, only needs User management right on the domain
* **technicalUserPassword** (string): Password of the technical account
* **active** (boolean): easy way to activate or deactivate a profile, if false the user of the appiId and appToken won't be able to login anymore
* **async** (boolean): 
  * **true**: Will complete the action immediately
  * **false**: Will use an event handler to complete the action 
* **asyncEventHandler** (string): Name of the event handler that will be used (see event handler section below)
* **asyncEventHandlerParams** (Dictionary<string, object>): Parameters that will be passed to the event handler

# Event handlers

If the parameter async is set to true, when an action completes it will try to trigger the action defined in the eventHandler from the parameter asyncEventHandler.
> For example: When an upload finishes it will try to load the custom DLL via a name (***asyncEventHandler***) and try to launch the event ***Uploaded*** with the event handler parameters (***asyncEventHandlerParams***).


# Examples

App id = f33c398c-0f77-4351-9f92-1e20fa3fd2f8

App token =  e1320735-e174-4150-9edb-b5daf85be6d1

Associated archive profiles metadata = ClientId, CustomerId, Documenttype

> Metadata names are case sensitive



### Authenticate

```shell
curl --location --request POST 'https://.../EAS.INTEGRATOR.API/service/authenticate' \
--header 'Accept: application/json' \
--header 'Content-Type: application/json' \
--data-raw '{"appId":"f33c398c-0f77-4351-9f92-1e20fa3fd2f8","appToken":"e1320735-e174-4150-9edb-b5daf85be6d1","accountName":"demoAccount"}'
```



### Upload document 

ClientId = 1, CustomerId = 1 and Documenttype = Invoice

```shell
curl --location --request POST 'https://.../EAS.INTEGRATOR.API/eas/documents' \
--header 'Authorization: Bearer M1oxTHBidko5Y3paR3lsM2tDaFp5RkViY29NUnJsTXBrZnpSd1ZaeVFIdWk4Y0ZnR21WbGJwbGlvRVBJMlp0am8wM3dZVzlrcXpoeGZHNHdiNVNBbFJEMDlaTFJ6ZnlBQUZweUhLeG9aclAlMkJCblglMkJCa2pJWmJJSUpMTyUyRkpNODRvNXdBMHJMOUI3NiUyQiUyQllMOWdTZlk0a1oydGpZQ1hNaGJpclRQdkRSaGZHSTVzS2R5dlZodktEY3NIaURja055dmhGalc0VjN5S0xkUEJpc3drYXFYYjkwd004eHlkRVRJTzc3MW12S0tuS3Jab1VMRVo5VkkxWHZNSm1IUDlLZCUyQmlpVWIlMkJvTWVUOXMlM0Q=' \
--form 'document=@"invoice20200201.pdf"' \
--form 'fingerprint=""' \
--form 'fingerprintAlgorithm="none"' \
--form 'metadata="[{name: \"ClientId\", value: \"1\"}, {name: \"CustomerId\", value: \"1\"},{name: \"Documenttype\", value: \"Invoice\"}]"'
```



### Get document

```shell
curl --location --request GET 'https://.../EAS.INTEGRATOR.API/eas/documents/A_FCBDAA9949D04B2980041F98E35AD05C_1' \
--header 'Accept: application/json' \
--header 'Authorization: Bearer M1oxTHBidko5Y3paR3lsM2tDaFp5RkViY29NUnJsTXBrZnpSd1ZaeVFIdWk4Y0ZnR21WbGJwbGlvRVBJMlp0am8wM3dZVzlrcXpoeGZHNHdiNVNBbFJEMDlaTFJ6ZnlBQUZweUhLeG9aclAlMkJCblglMkJCa2pJWmJJSUpMTyUyRkpNODRvNXdBMHJMOUI3NiUyQiUyQllMOWdTZlk0a1oydGpZQ1hNaGJpclRQdkRSaGZHSTVzS2R5dlZodktEY3NIaURja055dmhGalc0VjN5S0xkUEJpc3drYXFYYjkwd004eHlkRVRJTzc3MW12S0tuS3Jab1VMRVo5VkkxWHZNSm1IUDlLZCUyQmlpVWIlMkJvTWVUOXMlM0Q='
```


### Get content of an archived object

```shell
curl --location --request GET 'https://.../EAS.INTEGRATOR.API/eas/documents/A_FCBDAA9949D04B2980041F98E35AD05C_1?fileName=myFile1.pdf' \
--header 'Accept: application/json' \
--header 'Authorization: Bearer M1oxTHBidko5Y3paR3lsM2tDaFp5RkViY29NUnJsTXBrZnpSd1ZaeVFIdWk4Y0ZnR21WbGJwbGlvRVBJMlp0am8wM3dZVzlrcXpoeGZHNHdiNVNBbFJEMDlaTFJ6ZnlBQUZweUhLeG9aclAlMkJCblglMkJCa2pJWmJJSUpMTyUyRkpNODRvNXdBMHJMOUI3NiUyQiUyQllMOWdTZlk0a1oydGpZQ1hNaGJpclRQdkRSaGZHSTVzS2R5dlZodktEY3NIaURja055dmhGalc0VjN5S0xkUEJpc3drYXFYYjkwd004eHlkRVRJTzc3MW12S0tuS3Jab1VMRVo5VkkxWHZNSm1IUDlLZCUyQmlpVWIlMkJvTWVUOXMlM0Q='
```


### Get documents metadata

```shell
curl --location --request GET 'https://.../EAS.INTEGRATOR.API/eas/documents/A_FCBDAA9949D04B2980041F98E35AD05C_1/metadata' \
--header 'Accept: application/json' \
--header 'Content-Type: application/json' \
--header 'Authorization: Bearer M1oxTHBidko5Y3paR3lsM2tDaFp5RkViY29NUnJsTXBrZnpSd1ZaeVFIdWk4Y0ZnR21WbGJwbGlvRVBJMlp0am8wM3dZVzlrcXpoeGZHNHdiNVNBbFJEMDlaTFJ6ZnlBQUZweUhLeG9aclAlMkJCblglMkJCa2pJWmJJSUpMTyUyRkpNODRvNXdBMHJMOUI3NiUyQiUyQllMOWdTZlk0a1oydGpZQ1hNaGJpclRQdkRSaGZHSTVzS2R5dlZodktEY3NIaURja055dmhGalc0VjN5S0xkUEJpc3drYXFYYjkwd004eHlkRVRJTzc3MW12S0tuS3Jab1VMRVo5VkkxWHZNSm1IUDlLZCUyQmlpVWIlMkJvTWVUOXMlM0Q='
```



### Update documents meta (change ClientId into 2)

Change ClientId into 2.

```shell
curl --location --request PATCH 'https://.../EAS.INTEGRATOR.API/eas/documents/A_FCBDAA9949D04B2980041F98E35AD05C_1/metadata' \
--header 'Authorization: Bearer M1oxTHBidko5Y3paR3lsM2tDaFp5RkViY29NUnJsTXBrZnpSd1ZaeVFIdWk4Y0ZnR21WbGJwbGlvRVBJMlp0am8wM3dZVzlrcXpoeGZHNHdiNVNBbFJEMDlaTFJ6ZnlBQUZweUhLeG9aclAlMkJCblglMkJCa2pJWmJJSUpMTyUyRkpNODRvNXdBMHJMOUI3NiUyQiUyQllMOWdTZlk0a1oydGpZQ1hNaGJpclRQdkRSaGZHSTVzS2R5dlZodktEY3NIaURja055dmhGalc0VjN5S0xkUEJpc3drYXFYYjkwd004eHlkRVRJTzc3MW12S0tuS3Jab1VMRVo5VkkxWHZNSm1IUDlLZCUyQmlpVWIlMkJvTWVUOXMlM0Q=' \
--header 'Content-Type: application/json' \
--data-raw '{"metaData":[{"name":"ClientId","value": "2"}]}'
```



### Delete document

```shell
curl --location --request DELETE 'https://.../EAS.INTEGRATOR.API/eas/documents/A_FCBDAA9949D04B2980041F98E35AD05C_1?motivation=ReasonWhyWeDelete' \
--header 'Authorization: Bearer M1oxTHBidko5Y3paR3lsM2tDaFp5RkViY29NUnJsTXBrZnpSd1ZaeVFIdWk4Y0ZnR21WbGJwbGlvRVBJMlp0am8wM3dZVzlrcXpoeGZHNHdiNVNBbFJEMDlaTFJ6ZnlBQUZweUhLeG9aclAlMkJCblglMkJCa2pJWmJJSUpMTyUyRkpNODRvNXdBMHJMOUI3NiUyQiUyQllMOWdTZlk0a1oydGpZQ1hNaGJpclRQdkRSaGZHSTVzS2R5dlZodktEY3NIaURja055dmhGalc0VjN5S0xkUEJpc3drYXFYYjkwd004eHlkRVRJTzc3MW12S0tuS3Jab1VMRVo5VkkxWHZNSm1IUDlLZCUyQmlpVWIlMkJvTWVUOXMlM0Q='
```



### Get documents

Get documents for ClientId = 1:

> ClientId[eq]1

```shell
curl --location --request GET 'https://.../EAS.INTEGRATOR.API/eas/documents?fields=ClientId,CustomerId,Documenttype&filter=ClientId[eq]1&pageNumber=1&pageSize=20&sortBy=' \
--header 'Accept: application/json' \
--header 'Authorization: Bearer M1oxTHBidko5Y3paR3lsM2tDaFp5RkViY29NUnJsTXBrZnpSd1ZaeVFIdWk4Y0ZnR21WbGJwbGlvRVBJMlp0am8wM3dZVzlrcXpoeGZHNHdiNVNBbFJEMDlaTFJ6ZnlBQUZweUhLeG9aclAlMkJCblglMkJCa2pJWmJJSUpMTyUyRkpNODRvNXdBMHJMOUI3NiUyQiUyQllMOWdTZlk0a1oydGpZQ1hNaGJpclRQdkRSaGZHSTVzS2R5dlZodktEY3NIaURja055dmhGalc0VjN5S0xkUEJpc3drYXFYYjkwd004eHlkRVRJTzc3MW12S0tuS3Jab1VMRVo5VkkxWHZNSm1IUDlLZCUyQmlpVWIlMkJvTWVUOXMlM0Q='
```

Get documents for ClientId = 1 and Documenttype starts with Inv

> ClientId[eq]1,Documenttype[sw]Inv

```shell
curl --location --request GET 'https://.../EAS.INTEGRATOR.API/eas/documents?fields=ClientId,CustomerId,Documenttype&filter=ClientId[eq]1,Documenttype[sw]Inv&pageNumber=1&pageSize=20&sortBy=' \
--header 'Accept: application/json' \
--header 'Authorization: Bearer M1oxTHBidko5Y3paR3lsM2tDaFp5RkViY29NUnJsTXBrZnpSd1ZaeVFIdWk4Y0ZnR21WbGJwbGlvRVBJMlp0am8wM3dZVzlrcXpoeGZHNHdiNVNBbFJEMDlaTFJ6ZnlBQUZweUhLeG9aclAlMkJCblglMkJCa2pJWmJJSUpMTyUyRkpNODRvNXdBMHJMOUI3NiUyQiUyQllMOWdTZlk0a1oydGpZQ1hNaGJpclRQdkRSaGZHSTVzS2R5dlZodktEY3NIaURja055dmhGalc0VjN5S0xkUEJpc3drYXFYYjkwd004eHlkRVRJTzc3MW12S0tuS3Jab1VMRVo5VkkxWHZNSm1IUDlLZCUyQmlpVWIlMkJvTWVUOXMlM0Q='
```