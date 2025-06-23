/* eslint-disable @typescript-eslint/no-unused-vars */
import React, { ReactNode, useState } from "react";

import { getServerUrl, FTML,initialize } from "@kwarc/ftml-viewer";
import { FTMLDocument, FTMLFragment, FTMLSetup } from "./index";

export default {
  title: "Full Test",
};

await initialize("https://mathhub.info",true);

export const Complete = () => {
  console.log("Server URL according to leptos:", getServerUrl());
  const doc = {
    type: "FromBackend",
    uri: "https://mathhub.info/:sTeX?a=sTeX/MathTutorial&d=textbook&l=en",
    toc: "GET"
  }  as FTML.DocumentOptions;
  const frag1 = {
    type: "FromBackend",
    uri: "https://mathhub.info/:sTeX?a=sTeX/DemoExamples&d=problemtest&l=en&e=problem_1",
  } as FTML.FragmentOptions;
  const frag2 = {
    type: "FromBackend",
    uri: "https://mathhub.info/:sTeX?a=sTeX/DemoExamples&d=problemtest&l=en&e=problem_3",
  } as FTML.FragmentOptions;
  return (
    <div className="NARF" style={{width: "100vw",height: "100vh",maxHeight:"100vh",overflow:"scroll"}}>
      <FTMLSetup>
        <h1>React & FTML</h1>
        <div className="card">
          <Click />
        </div>
        <p>Here is a problem:</p>
        <FTMLFragment fragment={frag1} />
        <p>And here is one that logs every interaction to the console:</p>
        <FTMLFragment fragment={frag2} onProblem={(e) => console.log(e)} />
        <p> And here is a full document:</p>
        <FTMLDocument
          document={doc}
          onSectionTitle={(uri, _lvl) => <SectionTitle sec={uri} />}
          onFragment= {(uri, lvl) => {
            if (lvl.type === "Section") {return (ch) => (
            <SectionWrap uri={uri}>{ch}</SectionWrap>
            )}
          }}
        />
      </FTMLSetup>
    </div>
  );
};

const SectionTitle: React.FC<{ sec: string }> = ({ sec }) => {
  return (
    <div style={{ textAlign: "center" }}>
      <p>Here's a clicker thingy for {sec}:</p>
      <Click />
    </div>
  );
};

const Click: React.FC = () => {
  const [count, setCount] = useState(0);
  return (
    <>
      <button onClick={() => setCount((count) => count + 1)}>
        count is {count}
      </button>
      <p>Foo Bar</p>
    </>
  );
};

const SectionWrap: React.FC<{ uri: string; children: ReactNode }> = ({
  uri,
  children,
}) => {
  return (
    <div
      style={{ border: "1px solid red", margin: "1em 0", width: "calc(100%)" }}
    >
      <div style={{ textAlign: "center" }}>
        <p>This is the start of a section: {uri}!</p>
      </div>
      {children}
      <div style={{ textAlign: "center" }}>
        <p>This is the end of a section!</p>
      </div>
    </div>
  );
};
