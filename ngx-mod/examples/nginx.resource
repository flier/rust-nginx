*** Settings ***
Library    OperatingSystem
Library    Process

*** Variables ***
${NGINX_DIR}           /usr/local/nginx
${NGINX_ETC_DIR}       ${NGINX_DIR}/etc
${NGINX_BIN}           ${NGINX_DIR}/sbin/nginx
${NGINX_LOG_DIR}       ${NGINX_DIR}/logs
${NGINX_ACCESS_LOG}    ${NGINX_DIR}/logs/access.log
${NGINX_ERROR_LOG}     ${NGINX_DIR}/logs/error.log

*** Keywords ***
Start Nginx Process
    [Arguments]    ${conf}

    ${process} =    Start Process    ${NGINX_BIN}    -c    ${conf}

    Wait Until Keyword Succeeds    5s    1s    Process Should Be Running

    RETURN    ${process.pid}

Stop Nginx Process
    [Arguments]    ${conf}

    ${result} =        Run Process     ${NGINX_BIN}    -c    ${conf}    -s    stop
    Should Be Equal    ${result.rc}    ${0}

    Wait Until Keyword Succeeds    5s    1s    Process Should Be Stopped

Validate Nginx Configuration
    [Arguments]    ${conf}

    ${result} =        Run Process     ${NGINX_BIN}    -c    ${conf}    -t
    Should Be Equal    ${result.rc}    ${0}

Read Error Log
    [Arguments]    ${log_name}=error.log    ${log_file}=${NGINX_LOG_DIR}/${log_name}

    ${content} =    Get File    ${log_file}    encoding=SYSTEM

    RETURN    ${content}

Delete Error Log
    [Arguments]    ${log_name}=error.log    ${log_file}=${NGINX_LOG_DIR}/${log_name}

    Remove File    ${log_file}
