*** Settings ***
Documentation    setup code to write an upstream filter or load balancer.
Library          OperatingSystem
Library          RequestsLibrary
Resource         ./nginx.resource
Test Setup       Start Nginx Process
Test Teardown    Stop Nginx Process

*** Variables ***
${CONF}    ${NGINX_ETC_DIR}/httporigdst.conf

*** Test Cases ***
HttpOrigDst Module
    ${content} =    nginx.Read Error Log

    Should Contain    ${content}    httporigdst: init module    msg=httporigdst module should be loaded

    ${response} =    Wait Until Keyword Succeeds    5x                         1s
    ...              Get                            http://localhost:15502/    expected_status=anything    msg=Request to the external address

    ${content} =    nginx.Read Error Log

    Should Contain    ${content}    httporigdst: context not found    msg=httporigdst variable should be parsed

    IF    ${response.status_code} >= 500
        Should Contain    ${content}    httporigdst: orig_dst_addr same to local address    msg=Protocol not available
    ELSE
        Should Contain    ${content}    httporigdst: saving addr    msg=Internal address is ready
    END

*** Keywords ***
Start Nginx Process
    nginx.Validate Nginx Configuration    ${CONF}
    nginx.Start Nginx Process             ${CONF}

Stop Nginx Process
    nginx.Stop Nginx Process    ${CONF}
