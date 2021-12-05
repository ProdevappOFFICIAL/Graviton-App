import React from "react";
import { useRef } from "react";
import { useSetRecoilState } from "recoil";
import styled from "styled-components";
import { prompt } from "../utils/atoms";
import WindowBackground from "./window_background";

const StyledPromptOption = styled.div`
  color: white;
  margin: 10px;
  padding: 8px 10px;
  border-radius: 5px;
  font-size: 14px;
  background: ${({ theme }) => theme.elements.prompt.option.background};
  &:hover {
    cursor: pointer;
    background: ${({ theme }) => theme.elements.prompt.option.hover.background};
  }
`;

const StyledPrompt = styled.div`
  user-select: none;
  top: 0;
  left: 0;
  position: fixed;
  width: 100%;
  height: 100%;
  overflow: hidden;
  display: flex;
  justify-content: center;
  & .prompt {
    border: 1px solid ${({ theme }) => theme.elements.prompt.container.border};
    margin-top: 20px;
    width: 300px;
    border-radius: 10px;
    height: 200px;
    background: ${({ theme }) => theme.elements.prompt.container.background};
    box-shadow: 0px 2px 10px rgba(0, 0, 0, 0.15);
  }
`;

export interface OptionUtils {
  closePrompt: () => void;
}
export interface Option {
  label: string;
  onSelected: (utils: OptionUtils) => void;
}

interface PromptOptions {
  options: Option[];
}

export default function PromptContainer({ options }: PromptOptions) {
  const refBackground = useRef(null);
  const setPromptState = useSetRecoilState(prompt);

  function closePrompt(event: any) {
    if (event.target === refBackground.current) {
      setPromptState(null);
    }
  }

  return (
    <>
      <WindowBackground />
      <StyledPrompt onClick={closePrompt} ref={refBackground}>
        <div className="prompt">
          {options.map((option) => {
            function optionSelected() {
              option.onSelected({
                closePrompt: () => {
                  setPromptState(null);
                },
              });
            }

            return (
              <StyledPromptOption key={option.label} onClick={optionSelected}>
                {option.label}
              </StyledPromptOption>
            );
          })}
        </div>
      </StyledPrompt>
    </>
  );
}