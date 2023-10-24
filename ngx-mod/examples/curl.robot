*** Settings ***
Documentation    blocks HTTP requests if `user-agent` header starts with `curl`.
Library          RequestsLibrary
Resource         ./nginx.resource
Test Setup       Start Nginx Process
Test Teardown    Stop Nginx Process

*** Variables ***
${CONF}       ${NGINX_ETC_DIR}/curl.conf
&{CURL_UA}    user-agent=curl

*** Test Cases ***
CURL Module
    ${content} =    nginx.Read Error Log

    Should Contain    ${content}    CURL init module    msg=CURL module should be loaded
    Should Contain    ${content}    CURL set enable     msg=`curl` directive should be setted

    Wait Until Keyword Succeeds    5x                    1s
    ...                            GET                   http://localhost:15503/               expected_status=403
    ...                            headers=&{CURL_UA}    msg=CURL request should be blocked

    GET    http://localhost:15503/    expected_status=200    msg=Normal request should be passed

*** Keywords ***
Start Nginx Process
    nginx.Validate Nginx Configuration    ${CONF}
    nginx.Start Nginx Process             ${CONF}

Stop Nginx Process
    nginx.Stop Nginx Process    ${CONF}
