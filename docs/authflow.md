# Uni side
Start by trying to get access to request https://exampapers.ed.ac.uk/ to get a SAML auth redirect 
```
HTTP/1.1 302 Found
Set-Cookie: _opensaml_req_...=...; path=/; SameSite=None; Secure; HttpOnly; SameSite=None
Location: https://idp.ed.ac.uk/idp/profile/SAML2/Redirect/SSO?SAMLRequest=...&RelayState=...
Set-Cookie: RCKBMHKB=...; path=/; SameSite=None; Secure
```

Logging our request with the identity provider forwards us onto .../SSO?execution 
```
HTTP/1.1 302 302
Set-Cookie: __Host-JSESSIONID=...; Path=/; Secure; HttpOnly
Location: /idp/profile/SAML2/Redirect/SSO?execution=e1s1
Set-Cookie: RCKBMHKB=...; path=/; SameSite=None; Secure
```

Which, inturn redirects us to start a conversation 
```
HTTP/1.1 302 302
Location: /idp/profile/Authn/SAML2/POST/SSO/start?conversation=e1s1
```

This finally hands us off to Microsoft
```
HTTP/1.1 302 302
Location: https://login.microsoftonline.com/2e9f06b0-1669-4589-8789-10a06934dc61/saml2?SAMLRequest=...&RelayState=e1s1&whr=ed.ac.uk
```

MS then sets some cookies and forwards us back to the uni to enter our login information 
```
HTTP/1.1 302 Found
Location: https://edadfed.ed.ac.uk/adfs/ls/?client-request-id=...&username=&wa=wsignin1.0&wtrealm=...&wctx=...
Set-Cookie: buid=...
Set-Cookie: fpc=...
Set-Cookie: esctx=...
Set-Cookie: x-ms-gateway-slice=estsfd; ...
Set-Cookie: stsservicecookie=estsfd; ...

...
```

The uni then prompts us for login information (obs cutting down the HTML) 
```
HTTP/1.1 200 OK

<form method="post" id="loginForm" onKeyPress="Login.submitLoginRequest();" action="/adfs/ls/?client-request-id=...&username=&wa=wsignin1.0&wtrealm=..." >
    <input id="userNameInput" name="UserName" type="email" />
    <input id="passwordInput" name="Password" type="password" />
    <input type="checkbox" name="Kmsi" id="kmsiInput" value="true" />
    <label for="kmsiInput">Keep me signed in</label>
    <span id="submitButton" class="submit"  onKeyPress="Login.submitLoginRequest();"  onclick="Login.submitLoginRequest();">
    <input id="optionForms" type="hidden" name="AuthMethod" value="FormsAuthentication"/>
</form>

 <div id="authOptions">
     <form id="options" method="post" action="https://edadfed.ed.ac.uk:443/adfs/ls/?client-request-id=...&username=&wa=wsignin1.0&wtrealm=urn%3afederation%3aMicrosoftOnline&wctx=...">
        <script type="text/javascript">
        function SelectOption(option) {
            var w = document.getElementById('waitingWheelDiv');
            if(w) w.style.display = 'inline';
            var i = document.getElementById('optionSelection');
            i.value = option;
            document.forms['options'].submit();
            return false;
        }
        </script>
        <input id="optionSelection" type="hidden" name="AuthMethod" />
        <input id="userNameInputOptionsHolder" name="UserName" value="" type="hidden"/>
</form>
<script type="text/javascript">
    Login.submitLoginRequest = function () { 
        //... validate
        document.forms['loginForm'].submit();
        return false;
    };
</script>
```
this then sends the following 
```
POST /adfs/ls/?client-request-id=...&username=&wa=wsignin1.0&wtrealm=... HTTP/1.1
Host: edadfed.ed.ac.uk

UserName=...&Password=...&AuthMethod=FormsAuthentication
```
responded by 
```
HTTP/1.1 302 Found
Set-Cookie: MSISAuth=... 
Location: https://edadfed.ed.ac.uk/adfs/ls/?client-request-id=...&username=&wa=wsignin1.0&wtrealm=...
```

to give us all out msi auth info that is then posted to microsoft in a self submitting fourm (js submitts itself)
```
HTTP/1.1 200 OK
Set-Cookie: MSISAuth=...
Set-Cookie: MSISAuth=;... 
Set-Cookie: MSISSignOut=...
Set-Cookie: MSISAuthenticated=...
Set-Cookie: MSISLoopDetectionCookie=...

<form method="POST" name="hiddenform" action="https://login.microsoftonline.com:443/login.srf">
<input type="hidden" name="wa" value="wsignin1.0" />
<input type="hidden" name="wresult" value=...  />
<input type="submit" value="Submit" /></form>
```

# Were into MS land 

This is where parsing is going to cause problems. We've got to half blind navigate the auth flow
```
HTTP/1.1 200 OK
Set-Cookie: csrfspeedbump=...
Set-Cookie: ESTSAUTHPERSISTENT=...
Set-Cookie: ESTSAUTH=...
Set-Cookie: ESTSAUTHLIGHT=...
Set-Cookie: buid=...
Set-Cookie: esctx-...
Set-Cookie: fpc=; ...
Set-Cookie: x-ms-gateway-slice=; ...

```

### Do you trust ed.ac.uk popup 
We will have a form with the action `/appverify` and the continue button off 

```
<input type="submit" id="idSIButton9" class="win-button button_primary high-contrast-overrides button ext-button primary ext-primary" data-report-event="Signin_Submit" data-report-trigger="click" data-report-value="Submit" data-bind="
                attr: primaryButtonAttributes,
                css: { 'high-contrast-overrides': true },
                externalCss: {
                    'button': true,
                    'primary': true },
                value: primaryButtonText() || str['CT_PWD_STR_SignIn_Button_Next'],
                hasFocus: focusOnPrimaryButton,
                click: svr.fEnableLivePreview ?  function() { } : primaryButton_onClick,
                clickBubble: !svr.fEnableLivePreview,
                enable: isPrimaryButtonEnabled,
                visible: isPrimaryButtonVisible,
                preventTabbing: primaryButtonPreventTabbing" aria-describedby="appConfirmTitle appConfirmDescription" value="Continue" data-report-attached="1">
```


### Choose option / couldn't sign in 
If the browser try's to use the built in sec keys it'll fail to render and we will get a we couldn't sign in we can then attempt to choose a diff option sign-in another way, which forwards us to the same page as choosing which of the options we want to do 
```
<div id="loginHeader" class="row title ext-title" role="heading" aria-level="1" data-bind="text: enableAwpErrorFlag ? str['CT_FIDO_STR_Error_Page_Title'] : str['CT_FIDO_STR_Page_PasskeyError_Title'], 
        externalCss: { 'title': true }">We couldn't sign you in</div>
```

```
<a id="idA_PWD_SwitchToCredPicker" href="#" data-bind="
        text: isUserKnown ? str['CT_PWD_STR_SwitchToCredPicker_Link'] : str['CT_PWD_STR_SwitchToCredPicker_Link_NoUser'],
        ariaDescribedBy: ariaDescribedBy,
        click: switchToCredPicker_onClick,
        hasFocusEx: setFocus">Sign in another way</a>
```

I'm fairly sure that its the hidden input of login option


## Verification option (this might be bypassed if it chooses for you or somehow you got passed IT to not require auth)

Can be detected by a form with action `https://login.microsoftonline.com/common/SAS/ProcessAuth` we ideally want to choose PhoneNotifcation 
IT APPEARS AS THOUGH WE CAN CHOOSE IT EVEN IF ITS NOT LISTED BY HOOKING THE POST REQUEST

```
POST /common/SAS/BeginAuth HTTP/1.1
Host: login.microsoftonline.com
Cookie: ... 
Canary: ...
Client-Request-Id: ... 

{"AuthMethodId":"PhoneAppNotification","Method":"BeginAuth","ctx":"..."}
```

## Phone notification verification 
The user should then get a notification to approve a number, we can then relay the number back out 

`<div tabindex="0" aria-labelledby="idDiv_SAOTCAS_Description idRichContext_DisplaySign" id="idRichContext_DisplaySign" class="displaySign display-sign-height" data-bind="text: displaySign, hasFocusEx: focusOnSign(), css: { 'display-sign-height': svr.fEnableCenterFocusedApprovalNumber }">42</div>`

We want to do don't ask again 
```
<input id="idChkBx_SAOTCAS_TD" type="checkbox" value="true" data-bind="
                        attr: { name: svr.sTrustedDeviceCheckboxName },
                        ariaLabel: str['CT_SAOTCAS_STR_AddTD'],
                        ariaDescribedBy: ['idDiv_SAOTCAS_Title', 'idDiv_SAOTCAS_Description'].concat(description2 ? ['idDiv_RichContext_Description'] : []).join(' '),
                        hasFocusEx: tdCheckbox.isShown &amp;&amp; !focusOnSign(),
                        checked: tdCheckbox.isChecked,
                        disable: tdCheckbox.isDisabled" name="rememberMFA" aria-label="Don't ask again for 30 days" aria-describedby="idDiv_SAOTCAS_Title idDiv_SAOTCAS_Description">
```

Upon user filling it in we may be directed to 

## Stay signed in question
We will have a form tag that has an `action=/kmsi`
don't show again `<input id="KmsiCheckboxField" name="DontShowAgain" type="checkbox" value="true" data-bind="ariaLabel: str['STR_Kmsi_DontShowAgain']" aria-label="Don't show this again">`
and the yes that we want 
```
<input type="submit" id="idSIButton9" class="win-button button_primary high-contrast-overrides button ext-button primary ext-primary" data-report-event="Signin_Submit" data-report-trigger="click" data-report-value="Submit" data-bind="
                attr: primaryButtonAttributes,
                css: { 'high-contrast-overrides': true },
                externalCss: {
                    'button': true,
                    'primary': true },
                value: primaryButtonText() || str['CT_PWD_STR_SignIn_Button_Next'],
                hasFocus: focusOnPrimaryButton,
                click: svr.fEnableLivePreview ?  function() { } : primaryButton_onClick,
                clickBubble: !svr.fEnableLivePreview,
                enable: isPrimaryButtonEnabled,
                visible: isPrimaryButtonVisible,
                preventTabbing: primaryButtonPreventTabbing" aria-describedby="KmsiDescription" value="Yes" data-report-attached="1">

```

Please note that they're not the only fields in the submit they're just the ones we have to set


This should then auto flow through some backend verification before dumping us back out onto exam papers page 

TODO! Document the post kmsi button

# Touched domains 
```
edadfed.ed.ac.uk
exampapers.ed.ac.uk 
idp.ed.ac.uk 
login.live.com
login.microsoft.com
login.microsoftonline.com
```
Don't think we need to worry about these but 
```
aadcdn.msauth.net
aacdn.msftauth.net 
```
