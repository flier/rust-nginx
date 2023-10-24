*** Settings ***
Documentation    setup code to write an upstream filter or load balancer.
Library          OperatingSystem
Library          RequestsLibrary
Resource         ./nginx.resource
Test Setup       Start Nginx Process
Test Teardown    Stop Nginx Process

*** Variables ***
${CONF}    ${NGINX_ETC_DIR}/awssig.conf


*** Test Cases ***
Upstream Module
    ${content} =    nginx.Read Error Log

    Should Contain    ${content}    AwsSig init module        msg=AwsSig module should be loaded
    Should Contain    ${content}    AwsSig set enable         msg=`awssigv4` directive should be setted
    Should Contain    ${content}    AwsSig set access key     msg=`awssigv4_access_key` directive should be setted
    Should Contain    ${content}    AwsSig set secret key     msg=`awssigv4_secret_key` directive should be setted
    Should Contain    ${content}    AwsSig set S3 bucket      msg=`awssigv4_s3_bucket` directive should be setted
    Should Contain    ${content}    AwsSig set S3 endpoint    msg=`awssigv4_s3_endpoint` directive should be setted

    Wait Until Keyword Succeeds    5x     1s
    ...                            GET    http://localhost:15505/    expected_status=204    msg=Backend server is ready

    ${response} =    GET    http://localhost:15504/    expected_status=204    msg=AwsSig sign is ready

    Should Not Be Empty    ${response.headers['x-authorization']}
    Should Not Be Empty    ${response.headers['x-Amz-Date']}

    ${content} =      nginx.Read Error Log
    Should Contain    ${content}              AwsSig module    msg=AwsSig sign is working

*** Keywords ***
Start Nginx Process
    nginx.Validate Nginx Configuration    ${CONF}
    nginx.Start Nginx Process             ${CONF}

Stop Nginx Process
    nginx.Stop Nginx Process    ${CONF}
