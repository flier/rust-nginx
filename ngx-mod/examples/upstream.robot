*** Settings ***
Documentation    setup code to write an upstream filter or load balancer.
Library          OperatingSystem
Library          RequestsLibrary
Resource         ./nginx.resource
Test Setup       Start Nginx Process
Test Teardown    Stop Nginx Process

*** Variables ***
${CONF}    ${NGINX_ETC_DIR}/upstream.conf

*** Test Cases ***
Upstream Module
    ${content} =    nginx.Read Error Log

    Should Contain    ${content}    CUSTOM init module      msg=Custom directive should be setted
    Should Contain    ${content}    CUSTOM init upstream    msg=Upstream should be initialized

    Wait Until Keyword Succeeds    5x     1s
    ...                            GET    http://localhost:15501/    expected_status=418    msg=Backend server is ready

    GET    http://localhost:15500/    expected_status=418    msg=Upstream server is ready

    ${content} =    nginx.Read Error Log

    Should Contain    ${content}    CUSTOM init peer    msg=Peer should be initialized
    Should Contain    ${content}    CUSTOM get peer     msg=Peer should be ready
    Should Contain    ${content}    CUSTOM free peer    msg=Peer should be released

*** Keywords ***
Start Nginx Process
    nginx.Validate Nginx Configuration    ${CONF}
    nginx.Start Nginx Process             ${CONF}

Stop Nginx Process
    nginx.Stop Nginx Process    ${CONF}
