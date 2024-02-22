use immt_api::uris::DocumentURI;

#[test]
fn test() {
    let doc_uri = DocumentURI::new_unchecked("http://this?a=is&D&f=a&n=test");
    super::parse::HTMLParser::new(TEST, doc_uri).run();
}

static TEST: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>\t<meta charset="UTF-8">
\t<title>final.en</title>
\t<link rel="stylesheet" type="text/css" href="file:///home/jazzpirate/work/Software/sTeX/RusTeXNew/rustex/src/resources/rustex.css">
\t<link rel="stylesheet" href="https://cdn.jsdelivr.net/gh/dreampulse/computer-modern-web-font@master/font/Serif/cmun-serif.css">
</head>
<body>
<article class="rustex-body" style="font-size:18px;font-variant:normal;font-weight:normal;font-style:normal;font-family:Computer Modern Serif;--rustex-page-width:921.44238;line-height:1.2;--rustex-text-width:517.5;">
    <div class="rustex-vbox-container">
        <div class="rustex-vbox">
        </div>
    </div>
    <span shtml:sectionlevel="0" shtml:visible="false" style="display:none;">
        <div class="rustex-vskip" style="margin-bottom:0px;"></div>
    </span>
    <div class="rustex-scalewidth rustex-paragraph" style="font-size:83%;--rustex-scale-width:1.00;"><span><span shtml:sectionlevel="0" shtml:visible="false" style="display:none;"><div class="rustex-parindent" style="margin-left:22.5px"></div>&amp;#8205;</span> <div class="rustex-hskip" style="margin-left:0px;"></div></span></div>
    <a name="Doc-Start" id="Doc-Start">
    </a>
    <div class="rustex-vskip" style="margin-bottom:0px;"></div>
    <span>
        <div class="rustex-scalewidth rustex-paragraph" style="--rustex-scale-width:1.00;"><span>Name: <div class="rustex-hfill"></div>Matriculation Number:<div class="rustex-hskip" style="margin-left:85.35823px;"></div><div class="rustex-hskip" style="margin-left:0px;"></div><div class="rustex-hfil"></div> <div class="rustex-hskip" style="margin-left:0px;"></div></span></div>
        <div class="rustex-vskip" style="margin-bottom:7.74811px;"></div><div class="rustex-vskip" style="margin-bottom:-7.74811px;"></div><div class="rustex-vskip" style="margin-bottom:7.74811px;"></div><div class="rustex-vskip" style="margin-bottom:-7.74811px;"></div><div class="rustex-vskip" style="margin-bottom:15px;"></div>
        <div class="rustex-vskip" style="margin-bottom:0px;"></div>
        <div class="rustex-scalewidth rustex-paragraph" style="--rustex-scale-width:1.00;text-align:center;"><span><div class="rustex-hskip" style="margin-left:0px;"></div><div class="rustex-hskip" style="margin-left:0px;"></div><div class="rustex-hskip" style="margin-left:-7.5px;"></div><div class="rustex-hbox" style="max-width:0;"></div><div class="rustex-hskip" style="margin-left:7.5px;"></div><span style="font-size:120%;font-weight:bold;">Final Exam <div class="rustex-hskip" style="margin-left:0px;"></div></span></span></div>
        <div class="rustex-vskip" style="margin-bottom:0px;"></div><div class="rustex-vskip" style="margin-bottom:0px;"></div>
        <div class="rustex-scalewidth rustex-paragraph" style="font-size:120%;font-weight:bold;--rustex-scale-width:1.00;text-align:center;"><span>HeloProgForm (320201)</span></div>
        <div class="rustex-vskip" style="margin-bottom:0px;"></div><div class="rustex-vskip" style="margin-bottom:9.29672px;"></div>
        <div class="rustex-vskip" style="margin-bottom:0px;"></div>
        <div class="rustex-scalewidth rustex-paragraph" style="--rustex-scale-width:1.00;text-align:center;"><span>May ??., 2015</span></div>
        <div class="rustex-vskip" style="margin-bottom:0px;"></div><div class="rustex-vskip" style="margin-bottom:23.24432px;"></div><div class="rustex-vskip" style="margin-bottom:-23.24432px;"></div><div class="rustex-vskip" style="margin-bottom:23.24432px;"></div><div class="rustex-vskip" style="margin-bottom:-23.24432px;"></div><div class="rustex-vskip" style="margin-bottom:23.24432px;"></div><div class="rustex-vskip" style="margin-bottom:0px;"></div>
        <div class="rustex-scalewidth rustex-paragraph" style="--rustex-scale-width:1.00;"><span><div class="rustex-parindent" style="margin-left:22.5px"></div><span style="font-weight:bold;">You have two hours (sharp) for the test</span>;<div class="rustex-hfil"></div>Write the solutions to the sheet. <div class="rustex-hskip" style="margin-left:0px;"></div></span></div>
        <div class="rustex-vskip" style="margin-bottom:0px;"></div>
        <div class="rustex-scalewidth rustex-paragraph" style="--rustex-scale-width:1.00;"><span>The estimated time for solving this exam is 34 minutes, leaving you 86 minutes for revising your exam. <div class="rustex-hskip" style="margin-left:0px;"></div></span></div>
        <div class="rustex-vskip" style="margin-bottom:0px;"></div>
        <div class="rustex-scalewidth rustex-paragraph" style="--rustex-scale-width:1.00;"><span>You can reach 34 points if you solve all problems. You will only need 100 points for a perfect score, i.e. -66 points are bonus points. </span></div>
        <div class="rustex-vfill"></div><div class="rustex-vskip" style="margin-bottom:15px;"></div><div class="rustex-vskip" style="margin-bottom:0px;"></div>
        <div class="rustex-scalewidth rustex-paragraph" style="--rustex-scale-width:1.00;text-align:center;"><span><div class="rustex-hskip" style="margin-left:0px;"></div><div class="rustex-hskip" style="margin-left:0px;"></div><div class="rustex-hskip" style="margin-left:-7.5px;"></div><div class="rustex-hbox" style="max-width:0;"></div><div class="rustex-hskip" style="margin-left:7.5px;"></div>Different problems test different skills and knowledge, so do not get stuck on one problem. </span></div>
        <div class="rustex-vfill"></div><div class="rustex-vskip" style="margin-bottom:0px;"></div>
        <div class="rustex-scalewidth rustex-paragraph" style="--rustex-scale-width:1.00;text-align:center;"><span><div class="rustex-scalewidth rustex-hbox" style="--rustex-scale-width:1.00;justify-content:start;"><span><div class="rustex-hbox-container" style="height:185.2016px;"><div class="rustex-hbox"><span class="rustex-pdfmatrix" style="transform:matrix(1.58461,0,0,1.58461,0,0);"><div class="rustex-hbox" style="max-width:0;justify-content:start;"><div class="rustex-vcenter-container">
                                    <div>
                                        <table class="rustex-halign" style="--rustex-align-num:5;">
                                            <tbody>
                                                <tr>
                                                    <td class="rustex-noalign">
                                                        <div class="rustex-hrule" style="height:0.59999px;min-width:100%"><div style="background:#000000;height:0.59999px;" ></div></div>
                                                    </td>
                                                </tr>
                                                <tr>
                                                    <td class="rustex-halign-cell">
                                                        <div class="rustex-hbox"><div class="rustex-hkern" style="margin-left:-0.3px;"></div><div class="rustex-vrule" style="--rustex-this-width:0.59999px;align-self:stretch;background:#000000;" ></div><div class="rustex-hskip" style="margin-left:-0.3px;"></div><div class="rustex-hskip" style="margin-left:9px;"></div><div class="rustex-hfil"></div><div class="rustex-hskip" style="margin-left:9px;"></div><div class="rustex-hskip" style="margin-left:-0.3px;"></div><div class="rustex-vrule" style="--rustex-this-width:0.59999px;align-self:stretch;background:#000000;" ></div><div class="rustex-hkern" style="margin-left:-0.3px;"></div></div>
                                                    </td>
                                                    <td class="rustex-halign-cell" style="grid-column:span 3;">
                                                        <div class="rustex-hbox" style="justify-content:end;"><div class="rustex-hkern" style="margin-left:9.00002px;"></div><span style="font-size:67%;">To be used for grading, do not write here<div class="rustex-hfil"></div></span><div class="rustex-hskip" style="margin-left:9px;"></div><div class="rustex-hskip" style="margin-left:-0.3px;"></div><div class="rustex-vrule" style="--rustex-this-width:0.59999px;align-self:stretch;background:#000000;" ></div><div class="rustex-hskip" style="margin-left:-0.3px;"></div><div class="rustex-hskip" style="margin-left:3px;"></div><div class="rustex-hskip" style="margin-left:-0.3px;"></div><div class="rustex-vrule" style="--rustex-this-width:0.59999px;align-self:stretch;background:#000000;" ></div><div class="rustex-hskip" style="margin-left:-0.3px;"></div></div>
                                                    </td>
                                                    <td class="rustex-halign-cell">
                                                        <div class="rustex-hbox" style="justify-content:end;"><div class="rustex-hkern" style="margin-left:17.7px;"></div><div class="rustex-vrule" style="--rustex-this-width:0.59999px;align-self:stretch;background:#000000;" ></div><div class="rustex-hkern" style="margin-left:-0.3px;"></div></div>
                                                    </td>
                                                    <td class="rustex-halign-cell">
                                                    </td>
                                                    <td class="rustex-halign-cell">
                                                    </td>
                                                </tr>
                                                <tr>
                                                    <td class="rustex-noalign">
                                                        <div class="rustex-hrule" style="height:0.59999px;min-width:100%"><div style="background:#000000;height:0.59999px;" ></div></div>
                                                    </td>
                                                </tr>
                                                <tr>
                                                    <td class="rustex-halign-cell">
                                                        <div class="rustex-hbox"><div class="rustex-hkern" style="margin-left:-0.3px;"></div><div class="rustex-vrule" style="--rustex-this-width:0.59999px;align-self:stretch;background:#000000;" ></div><div class="rustex-hskip" style="margin-left:-0.3px;"></div><div class="rustex-hskip" style="margin-left:9px;"></div><div class="rustex-hskip" style="margin-left:0.00002px;"></div>prob.<div class="rustex-hfil"></div><div class="rustex-hskip" style="margin-left:9px;"></div><div class="rustex-hskip" style="margin-left:-0.3px;"></div><div class="rustex-vrule" style="--rustex-this-width:0.59999px;align-self:stretch;background:#000000;" ></div><div class="rustex-hkern" style="margin-left:-0.3px;"></div></div>
                                                    </td>
                                                    <td class="rustex-halign-cell">
                                                        <div class="rustex-hbox" style="justify-content:end;"><div class="rustex-hkern" style="margin-left:9.00002px;"></div>0.1<div class="rustex-hfil"></div><div class="rustex-hskip" style="margin-left:9px;"></div><div class="rustex-hskip" style="margin-left:-0.3px;"></div><div class="rustex-vrule" style="--rustex-this-width:0.59999px;align-self:stretch;background:#000000;" ></div><div class="rustex-hkern" style="margin-left:-0.3px;"></div></div>
                                                    </td>
                                                    <td class="rustex-halign-cell">
                                                        <div class="rustex-hbox" style="justify-content:end;"><div class="rustex-hkern" style="margin-left:9.00002px;"></div>0.2<div class="rustex-hfil"></div><div class="rustex-hskip" style="margin-left:9px;"></div><div class="rustex-hskip" style="margin-left:-0.3px;"></div><div class="rustex-vrule" style="--rustex-this-width:0.59999px;align-self:stretch;background:#000000;" ></div><div class="rustex-hkern" style="margin-left:-0.3px;"></div></div>
                                                    </td>
                                                    <td class="rustex-halign-cell">
                                                        <div class="rustex-hbox" style="justify-content:end;"><div class="rustex-hkern" style="margin-left:9.00002px;"></div>Sum<div class="rustex-hfil"></div><div class="rustex-hskip" style="margin-left:9px;"></div><div class="rustex-hskip" style="margin-left:-0.3px;"></div><div class="rustex-vrule" style="--rustex-this-width:0.59999px;align-self:stretch;background:#000000;" ></div><div class="rustex-hskip" style="margin-left:-0.3px;"></div><div class="rustex-hskip" style="margin-left:3px;"></div><div class="rustex-hskip" style="margin-left:-0.3px;"></div><div class="rustex-vrule" style="--rustex-this-width:0.59999px;align-self:stretch;background:#000000;" ></div><div class="rustex-hkern" style="margin-left:-0.3px;"></div></div>
                                                    </td>
                                                    <td class="rustex-halign-cell">
                                                        <div class="rustex-hbox"><div class="rustex-hkern" style="margin-left:9.00002px;"></div>grade<div class="rustex-hfil"></div><div class="rustex-hskip" style="margin-left:9px;"></div><div class="rustex-hskip" style="margin-left:-0.3px;"></div><div class="rustex-vrule" style="--rustex-this-width:0.59999px;align-self:stretch;background:#000000;" ></div><div class="rustex-hkern" style="margin-left:-0.3px;"></div></div>
                                                    </td>
                                                </tr>
                                                <tr>
                                                    <td class="rustex-noalign">
                                                        <div class="rustex-hrule" style="height:0.59999px;min-width:100%"><div style="background:#000000;height:0.59999px;" ></div></div>
                                                    </td>
                                                </tr>
                                                <tr>
                                                    <td class="rustex-halign-cell">
                                                        <div class="rustex-hbox"><div class="rustex-hkern" style="margin-left:-0.3px;"></div><div class="rustex-vrule" style="--rustex-this-width:0.59999px;align-self:stretch;background:#000000;" ></div><div class="rustex-hskip" style="margin-left:-0.3px;"></div><div class="rustex-hskip" style="margin-left:9px;"></div><div class="rustex-hskip" style="margin-left:0.00002px;"></div>total<div class="rustex-hfil"></div><div class="rustex-hskip" style="margin-left:9px;"></div><div class="rustex-hskip" style="margin-left:-0.3px;"></div><div class="rustex-vrule" style="--rustex-this-width:0.59999px;align-self:stretch;background:#000000;" ></div><div class="rustex-hkern" style="margin-left:-0.3px;"></div></div>
                                                    </td>
                                                    <td class="rustex-halign-cell">
                                                        <div class="rustex-hbox" style="justify-content:end;"><div class="rustex-hkern" style="margin-left:9.00002px;"></div>4<div class="rustex-hfil"></div><div class="rustex-hskip" style="margin-left:9px;"></div><div class="rustex-hskip" style="margin-left:-0.3px;"></div><div class="rustex-vrule" style="--rustex-this-width:0.59999px;align-self:stretch;background:#000000;" ></div><div class="rustex-hkern" style="margin-left:-0.3px;"></div></div>
                                                    </td>
                                                    <td class="rustex-halign-cell">
                                                        <div class="rustex-hbox" style="justify-content:end;"><div class="rustex-hkern" style="margin-left:9.00002px;"></div>30<div class="rustex-hfil"></div><div class="rustex-hskip" style="margin-left:9px;"></div><div class="rustex-hskip" style="margin-left:-0.3px;"></div><div class="rustex-vrule" style="--rustex-this-width:0.59999px;align-self:stretch;background:#000000;" ></div><div class="rustex-hkern" style="margin-left:-0.3px;"></div></div>
                                                    </td>
                                                    <td class="rustex-halign-cell">
                                                        <div class="rustex-hbox" style="justify-content:end;"><div class="rustex-hkern" style="margin-left:9.00002px;"></div>34<div class="rustex-hfil"></div><div class="rustex-hskip" style="margin-left:9px;"></div><div class="rustex-hskip" style="margin-left:-0.3px;"></div><div class="rustex-vrule" style="--rustex-this-width:0.59999px;align-self:stretch;background:#000000;" ></div><div class="rustex-hskip" style="margin-left:-0.3px;"></div><div class="rustex-hskip" style="margin-left:3px;"></div><div class="rustex-hskip" style="margin-left:-0.3px;"></div><div class="rustex-vrule" style="--rustex-this-width:0.59999px;align-self:stretch;background:#000000;" ></div><div class="rustex-hkern" style="margin-left:-0.3px;"></div></div>
                                                    </td>
                                                    <td class="rustex-halign-cell">
                                                        <div class="rustex-hbox" style="justify-content:end;"><div class="rustex-hkern" style="margin-left:17.7px;"></div><div class="rustex-vrule" style="--rustex-this-width:0.59999px;align-self:stretch;background:#000000;" ></div><div class="rustex-hkern" style="margin-left:-0.3px;"></div></div>
                                                    </td>
                                                </tr>
                                                <tr>
                                                    <td class="rustex-noalign">
                                                        <div class="rustex-hrule" style="height:0.59999px;min-width:100%"><div style="background:#000000;height:0.59999px;" ></div></div>
                                                    </td>
                                                </tr>
                                                <tr>
                                                    <td class="rustex-halign-cell">
                                                        <div class="rustex-hbox"><div class="rustex-hkern" style="margin-left:-0.3px;"></div><div class="rustex-vrule" style="--rustex-this-width:0.59999px;align-self:stretch;background:#000000;" ></div><div class="rustex-hskip" style="margin-left:-0.3px;"></div><div class="rustex-hskip" style="margin-left:9px;"></div><div class="rustex-hskip" style="margin-left:0.00002px;"></div>reached<div class="rustex-hfil"></div><div class="rustex-hskip" style="margin-left:9px;"></div><div class="rustex-hskip" style="margin-left:-0.3px;"></div><div class="rustex-vrule" style="--rustex-this-width:0.59999px;align-self:stretch;background:#000000;" ></div><div class="rustex-hkern" style="margin-left:-0.3px;"></div></div>
                                                    </td>
                                                    <td class="rustex-halign-cell">
                                                        <div class="rustex-hbox" style="justify-content:end;"><div class="rustex-hkern" style="margin-left:17.7px;"></div><div class="rustex-vrule" style="--rustex-this-width:0.59999px;align-self:stretch;background:#000000;" ></div><div class="rustex-hkern" style="margin-left:-0.3px;"></div></div>
                                                    </td>
                                                    <td class="rustex-halign-cell">
                                                        <div class="rustex-hbox" style="justify-content:end;"><div class="rustex-hkern" style="margin-left:17.7px;"></div><div class="rustex-vrule" style="--rustex-this-width:0.59999px;align-self:stretch;background:#000000;" ></div><div class="rustex-hkern" style="margin-left:-0.3px;"></div></div>
                                                    </td>
                                                    <td class="rustex-halign-cell">
                                                        <div class="rustex-hbox" style="justify-content:end;"><div class="rustex-hkern" style="margin-left:17.7px;"></div><div class="rustex-vrule" style="--rustex-this-width:0.59999px;align-self:stretch;background:#000000;" ></div><div class="rustex-hskip" style="margin-left:-0.3px;"></div><div class="rustex-hskip" style="margin-left:3px;"></div><div class="rustex-hskip" style="margin-left:-0.3px;"></div><div class="rustex-vrule" style="--rustex-this-width:0.59999px;align-self:stretch;background:#000000;" ></div><div class="rustex-hkern" style="margin-left:-0.3px;"></div></div>
                                                    </td>
                                                    <td class="rustex-halign-cell">
                                                        <div class="rustex-hbox" style="justify-content:end;"><div class="rustex-hkern" style="margin-left:17.7px;"></div><div class="rustex-vrule" style="--rustex-this-width:0.59999px;align-self:stretch;background:#000000;" ></div><div class="rustex-hkern" style="margin-left:-0.3px;"></div></div>
                                                    </td>
                                                </tr>
                                                <tr>
                                                    <td class="rustex-noalign">
                                                        <div class="rustex-hrule" style="height:0.59999px;min-width:100%"><div style="background:#000000;height:0.59999px;" ></div></div>
                                                    </td>
                                                </tr>
                                            </tbody>
                                        </table>
                                    </div>
                                </div></div></span></div></div><div class="rustex-hkern" style="margin-left:517.56165px;"></div></span></div></span></div>
        <div class="rustex-vskip" style="margin-bottom:0px;"></div><div class="rustex-vskip" style="margin-bottom:23.24432px;"></div><div class="rustex-vskip" style="margin-bottom:-23.24432px;"></div><div class="rustex-vskip" style="margin-bottom:23.24432px;"></div><div class="rustex-vskip" style="margin-bottom:-23.24432px;"></div><div class="rustex-vskip" style="margin-bottom:23.24432px;"></div><div class="rustex-vskip" style="margin-bottom:0px;"></div>
        <div class="rustex-scalewidth rustex-paragraph" style="--rustex-scale-width:1.00;"><span>Please consider the following rules; otherwise you may lose points: <div class="rustex-hskip" style="margin-left:0px;"></div></span></div>
        <div class="rustex-vskip" style="margin-bottom:15px;"></div><div class="rustex-vskip" style="margin-bottom:-15px;"></div><div class="rustex-vskip" style="margin-bottom:9px;"></div><div class="rustex-vskip" style="margin-bottom:6px;"></div>
        <div class="rustex-scalewidth rustex-paragraph" style="--rustex-scale-width:0.93;margin-left:37.5px;"><span><div class="rustex-hskip" style="margin-left:0px;"></div><div class="rustex-hskip" style="margin-left:-30px;"></div><div class="rustex-hskip" style="margin-left:-7.5px;"></div><div class="rustex-scalewidth rustex-hbox" style="--rustex-scale-width:0.06;justify-content:end;"><span>•</span></div><div class="rustex-hskip" style="margin-left:7.5px;"></div>“Prove or refute” means: If you think that the statement is correct, give a formal proof. If not, give a counter-example that makes it fail. <div class="rustex-hskip" style="margin-left:0px;"></div></span></div>
        <div class="rustex-vskip" style="margin-bottom:6px;"></div><div class="rustex-vskip" style="margin-bottom:6px;"></div>
        <div class="rustex-scalewidth rustex-paragraph" style="--rustex-scale-width:0.93;margin-left:37.5px;"><span><div class="rustex-hskip" style="margin-left:0px;"></div><div class="rustex-hskip" style="margin-left:-30px;"></div><div class="rustex-hskip" style="margin-left:-7.5px;"></div><div class="rustex-scalewidth rustex-hbox" style="--rustex-scale-width:0.06;justify-content:end;"><span>•</span></div><div class="rustex-hskip" style="margin-left:7.5px;"></div>Always justify your statements. Unless you are explicitly allowed to, do not just answer “yes” or “no”, but instead prove your statement or refer to an appropriate definition or theorem from the lecture. <div class="rustex-hskip" style="margin-left:0px;"></div></span></div>
        <div class="rustex-vskip" style="margin-bottom:0px;"></div><div class="rustex-vskip" style="margin-bottom:0px;"></div>
    </span>
    <div class="rustex-vskip" style="margin-bottom:6px;"></div>
    <div class="rustex-scalewidth rustex-paragraph" style="font-size:83%;--rustex-scale-width:0.93;margin-left:37.5px;"><span><div class="rustex-hskip" style="margin-left:0px;"></div><div class="rustex-hskip" style="margin-left:-30px;"></div><div class="rustex-hskip" style="margin-left:-7.5px;"></div><div class="rustex-scalewidth rustex-hbox" style="font-size:120%;--rustex-scale-width:0.06;justify-content:end;"><span>•</span></div><div class="rustex-hskip" style="margin-left:7.5px;"></div>If you write program code, give comments! <div class="rustex-hskip" style="margin-left:0px;"></div></span></div>
    <div class="rustex-vskip" style="margin-bottom:15px;"></div><div class="rustex-vfil"></div><div class="rustex-vskip" style="margin-bottom:15px;"></div><div class="rustex-vskip" style="margin-bottom:0px;"></div><div class="rustex-vskip" style="margin-bottom:0px;"></div>
    <div class="rustex-hbox" shtml:visible="false" shtml:doctitle="" style="font-size:83%;display:none;">Generalities</div>
    <span shtml:section="0" style="font-size:83%;">
        <div class="rustex-vskip" style="margin-bottom:0px;"></div>
        <div class="rustex-scalewidth rustex-paragraph" style="--rustex-scale-width:1.00;"><span><span shtml:sectiontitle=""><span shtml:visible="false" style="display:none;"> </span>Generalities<span shtml:visible="false" style="display:none;"> </span></span></span></div>
        <div class="rustex-vskip" style="margin-bottom:0px;"></div>
        <div class="rustex-scalewidth rustex-paragraph" style="--rustex-scale-width:1.00;"><span><span shtml:inputref="http://mathhub.info/HelloWorld/hwexam/problems/explain-hw" shtml:visible="false" style="display:none;">&amp;#8205;</span></span></div>
        <div class="rustex-vfil"></div><div class="rustex-vskip" style="margin-bottom:15px;"></div><div class="rustex-vskip" style="margin-bottom:0px;"></div><div class="rustex-vskip" style="margin-bottom:0px;"></div>
    </span>
    <span shtml:section="0" style="font-size:83%;">
        <div class="rustex-scalewidth rustex-paragraph" style="--rustex-scale-width:1.00;"><span><span shtml:sectiontitle=""><span shtml:visible="false" style="display:none;"> </span>Flexiformalizing in sTeX<span shtml:visible="false" style="display:none;"> </span></span></span></div>
        <div class="rustex-vskip" style="margin-bottom:0px;"></div>
        <div class="rustex-scalewidth rustex-paragraph" style="--rustex-scale-width:1.00;"><span><span shtml:visible="false" shtml:inputref="http://mathhub.info/HelloWorld/hwexam/problems/hello-stex-smglom" style="display:none;">&amp;#8205;</span></span></div>
        <div class="rustex-vfil"></div><div class="rustex-vskip" style="margin-bottom:15px;"></div><div class="rustex-vskip" style="margin-bottom:0px;"></div><div class="rustex-vskip" style="margin-bottom:0px;"></div>
    </span>
    <span style="font-size:83%;">
        <div class="rustex-scalewidth rustex-paragraph" style="--rustex-scale-width:1.00;"><span><div class="rustex-parindent" style="margin-left:22.5px"></div> </span></div>
        <div class="rustex-vfill"></div><div class="rustex-vskip" style="margin-bottom:15px;"></div><div class="rustex-vskip" style="margin-bottom:0px;"></div>
        <div class="rustex-scalewidth rustex-paragraph" style="--rustex-scale-width:1.00;text-align:center;"><span><div class="rustex-hskip" style="margin-left:0px;"></div><div class="rustex-hskip" style="margin-left:0px;"></div><div class="rustex-hskip" style="margin-left:-7.5px;"></div><div class="rustex-hbox" style="max-width:0;"></div><div class="rustex-hskip" style="margin-left:7.5px;"></div>This page was intentionally left blank for extra space</span></div>
        <div class="rustex-vskip" style="margin-bottom:15px;"></div><div class="rustex-vskip" style="margin-bottom:15px;"></div><div class="rustex-vskip" style="margin-bottom:0px;"></div><div class="rustex-vskip" style="margin-bottom:0px;"></div>
    </span>
    <span style="font-size:83%;">
        <div class="rustex-scalewidth rustex-paragraph" style="--rustex-scale-width:1.00;"><span><div class="rustex-parindent" style="margin-left:22.5px"></div> </span></div>
        <div class="rustex-vfill"></div><div class="rustex-vskip" style="margin-bottom:15px;"></div><div class="rustex-vskip" style="margin-bottom:0px;"></div>
        <div class="rustex-scalewidth rustex-paragraph" style="--rustex-scale-width:1.00;text-align:center;"><span><div class="rustex-hskip" style="margin-left:0px;"></div><div class="rustex-hskip" style="margin-left:0px;"></div><div class="rustex-hskip" style="margin-left:-7.5px;"></div><div class="rustex-hbox" style="max-width:0;"></div><div class="rustex-hskip" style="margin-left:7.5px;"></div>This page was intentionally left blank for extra space</span></div>
        <div class="rustex-vskip" style="margin-bottom:15px;"></div><div class="rustex-vskip" style="margin-bottom:15px;"></div><div class="rustex-vskip" style="margin-bottom:0px;"></div><div class="rustex-vskip" style="margin-bottom:0px;"></div>
    </span>
    <div class="rustex-vbox-container">
        <div class="rustex-vbox">
        </div>
    </div>
    <div class="rustex-vskip" style="margin-bottom:15px;"></div><div class="rustex-vskip" style="margin-bottom:0px;"></div><div class="rustex-vskip" style="margin-bottom:0px;"></div>
</article>
</body>
</html>"#;
