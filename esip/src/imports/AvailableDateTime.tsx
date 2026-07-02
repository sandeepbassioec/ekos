import svgPaths from "./svg-3zssg190q7";

function Frame1() {
  return (
    <div className="content-stretch flex flex-col items-start relative shrink-0">
      <p className="font-['Inter:Bold',sans-serif] font-bold leading-[24px] min-w-full not-italic relative shrink-0 text-[#4d4e53] text-[16px] w-[min-content]">{`Available Date & Time `}</p>
    </div>
  );
}

function LeftSection() {
  return (
    <div className="content-stretch flex gap-[8px] items-center relative shrink-0" data-name="Left Section">
      <Frame1 />
    </div>
  );
}

function LabelBody() {
  return (
    <div className="bg-white content-stretch flex gap-[2px] items-center justify-center not-italic px-[4px] relative rounded-[4px] shrink-0 text-[12px] whitespace-nowrap" data-name="Label Body">
      <p className="font-['Inter:Regular',sans-serif] font-normal leading-[16px] relative shrink-0 text-[#202020]">Set Date</p>
      <div className="flex flex-col font-['Inter:Bold',sans-serif] font-bold justify-center leading-[0] relative shrink-0 text-[#c23f2e] uppercase">
        <p className="leading-[16px]">*</p>
      </div>
    </div>
  );
}

function Label() {
  return (
    <div className="absolute content-stretch flex flex-col items-start left-0 px-[8px] top-[-12px] w-[312px] z-[3]" data-name="Label">
      <LabelBody />
    </div>
  );
}

function InputContainer() {
  return (
    <div className="bg-white relative rounded-[8px] shrink-0 w-full z-[2]" data-name="Input Container">
      <div aria-hidden="true" className="absolute border border-[#bfbfbf] border-solid inset-0 pointer-events-none rounded-[8px]" />
      <div className="flex flex-row items-center size-full">
        <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[8px] relative w-full">
          <div className="overflow-clip relative shrink-0 size-[24px]" data-name="Calendar Icon">
            <div className="absolute inset-[10%_15%_15%_15%]" data-name="Vector">
              <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16.8 18">
                <path d={svgPaths.p2c0d6100} fill="var(--fill-0, #4D4E53)" id="Vector" />
              </svg>
            </div>
          </div>
          <p className="font-['Inter:Regular',sans-serif] font-normal leading-[24px] not-italic relative shrink-0 text-[#656565] text-[16px] whitespace-nowrap">mm/dd/yyyy</p>
        </div>
      </div>
    </div>
  );
}

function LabelBody1() {
  return (
    <div className="bg-white content-stretch flex gap-[2px] items-center justify-center not-italic px-[4px] relative rounded-[4px] shrink-0 text-[12px] whitespace-nowrap" data-name="Label Body">
      <p className="font-['Inter:Regular',sans-serif] font-normal leading-[16px] relative shrink-0 text-[#202020]">Set Time</p>
      <div className="flex flex-col font-['Inter:Bold',sans-serif] font-bold justify-center leading-[0] relative shrink-0 text-[#c23f2e] uppercase">
        <p className="leading-[16px]">*</p>
      </div>
    </div>
  );
}

function Label1() {
  return (
    <div className="absolute content-stretch flex flex-col items-start left-0 px-[8px] top-[-12px] z-[2]" data-name="Label">
      <LabelBody1 />
    </div>
  );
}

function InputContainer1() {
  return (
    <div className="bg-white relative rounded-[8px] shrink-0 w-full z-[1]" data-name="Input Container">
      <div aria-hidden="true" className="absolute border border-[#bfbfbf] border-solid inset-0 pointer-events-none rounded-[8px]" />
      <div className="flex flex-row items-center size-full">
        <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[8px] relative w-full">
          <div className="overflow-clip relative shrink-0 size-[24px]" data-name="Calendar Icon">
            <div className="absolute inset-[10%]" data-name="Vector">
              <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 19.2 19.2">
                <path d={svgPaths.p361b2270} fill="var(--fill-0, #4D4E53)" id="Vector" />
              </svg>
            </div>
          </div>
          <p className="font-['Inter:Regular',sans-serif] font-normal leading-[24px] not-italic relative shrink-0 text-[#656565] text-[16px] whitespace-nowrap">HH:MM</p>
          <div className="overflow-clip relative shrink-0 size-[24px]" data-name="Calendar Icon">
            <div className="-translate-y-1/2 absolute aspect-[14/8] left-[23.32%] right-[18.35%] top-[calc(50%+0.4px)]" data-name="Vector">
              <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 14 8">
                <path d={svgPaths.p3c710100} fill="var(--fill-0, #4D4E53)" id="Vector" />
              </svg>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function LabelBody2() {
  return (
    <div className="bg-white content-stretch flex gap-[2px] items-center justify-center not-italic px-[4px] relative rounded-[4px] shrink-0 text-[12px] whitespace-nowrap" data-name="Label Body">
      <p className="font-['Inter:Regular',sans-serif] font-normal leading-[16px] relative shrink-0 text-[#202020]">AM/PM</p>
      <div className="flex flex-col font-['Inter:Bold',sans-serif] font-bold justify-center leading-[0] relative shrink-0 text-[#c23f2e] uppercase">
        <p className="leading-[16px]">*</p>
      </div>
    </div>
  );
}

function Label2() {
  return (
    <div className="absolute content-stretch flex flex-col items-start left-0 px-[8px] top-[-12px] z-[2]" data-name="Label">
      <LabelBody2 />
    </div>
  );
}

function InputContainer2() {
  return (
    <div className="bg-white relative rounded-[8px] shrink-0 w-full z-[1]" data-name="Input Container">
      <div aria-hidden="true" className="absolute border border-[#bfbfbf] border-solid inset-0 pointer-events-none rounded-[8px]" />
      <div className="flex flex-row items-center size-full">
        <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[8px] relative w-full">
          <p className="font-['Inter:Regular',sans-serif] font-normal leading-[24px] not-italic relative shrink-0 text-[#656565] text-[16px] whitespace-nowrap">AM</p>
          <div className="overflow-clip relative shrink-0 size-[24px]" data-name="Calendar Icon">
            <div className="-translate-y-1/2 absolute aspect-[14/8] left-[23.32%] right-[18.35%] top-[calc(50%+0.4px)]" data-name="Vector">
              <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 14 8">
                <path d={svgPaths.p3c710100} fill="var(--fill-0, #4D4E53)" id="Vector" />
              </svg>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function LabelBody3() {
  return (
    <div className="bg-white content-stretch flex gap-[2px] items-center justify-center not-italic px-[4px] relative rounded-[4px] shrink-0 text-[12px] whitespace-nowrap" data-name="Label Body">
      <p className="font-['Inter:Regular',sans-serif] font-normal leading-[16px] relative shrink-0 text-[#202020]">Timezone</p>
      <div className="flex flex-col font-['Inter:Bold',sans-serif] font-bold justify-center leading-[0] relative shrink-0 text-[#c23f2e] uppercase">
        <p className="leading-[16px]">*</p>
      </div>
    </div>
  );
}

function Label3() {
  return (
    <div className="absolute content-stretch flex flex-col items-start left-0 px-[8px] top-[-12px] z-[2]" data-name="Label">
      <LabelBody3 />
    </div>
  );
}

function InputContainer3() {
  return (
    <div className="bg-white relative rounded-[8px] shrink-0 w-full z-[1]" data-name="Input Container">
      <div aria-hidden="true" className="absolute border border-[#bfbfbf] border-solid inset-0 pointer-events-none rounded-[8px]" />
      <div className="flex flex-row items-center size-full">
        <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[8px] relative w-full">
          <p className="font-['Inter:Regular',sans-serif] font-normal leading-[24px] not-italic relative shrink-0 text-[#656565] text-[16px] whitespace-nowrap">EST</p>
          <div className="overflow-clip relative shrink-0 size-[24px]" data-name="Calendar Icon">
            <div className="-translate-y-1/2 absolute aspect-[14/8] left-[23.32%] right-[18.35%] top-[calc(50%+0.4px)]" data-name="Vector">
              <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 14 8">
                <path d={svgPaths.p3c710100} fill="var(--fill-0, #4D4E53)" id="Vector" />
              </svg>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function Frame() {
  return (
    <div className="content-stretch flex gap-[8px] items-start relative shrink-0">
      <div className="content-stretch flex flex-col gap-[4px] isolate items-start relative shrink-0" data-name="Time Filed">
        <Label1 />
        <InputContainer1 />
      </div>
      <div className="content-stretch flex flex-col gap-[4px] h-[40px] isolate items-start relative shrink-0" data-name="Time Filed">
        <Label2 />
        <InputContainer2 />
      </div>
      <div className="content-stretch flex flex-col gap-[4px] h-[40px] isolate items-start relative shrink-0" data-name="Time Filed">
        <Label3 />
        <InputContainer3 />
      </div>
    </div>
  );
}

function Body() {
  return (
    <div className="bg-white flex-[1_0_0] min-h-px min-w-px relative w-[500px]" data-name="Body">
      <div className="overflow-clip rounded-[inherit] size-full">
        <div className="content-stretch flex flex-col gap-[24px] items-start p-[24px] relative size-full">
          <div className="content-stretch flex flex-col gap-[4px] isolate items-start relative shrink-0 w-[354px]" data-name="Data field">
            <Label />
            <InputContainer />
          </div>
          <Frame />
        </div>
      </div>
    </div>
  );
}

export default function AvailableDateTime() {
  return (
    <div className="content-stretch flex flex-col items-start relative size-full" data-name="Available Date & Time">
      <div className="bg-white content-stretch flex items-center justify-between max-w-[500px] min-w-[500px] px-[16px] py-[12px] relative rounded-tl-[8px] rounded-tr-[8px] shrink-0 w-[500px]" data-name="Modal Elemets">
        <div aria-hidden="true" className="absolute border-[#eaeaea] border-b border-solid inset-0 pointer-events-none rounded-tl-[8px] rounded-tr-[8px]" />
        <LeftSection />
        <div className="overflow-clip relative shrink-0 size-[24px]" data-name="Close Icon">
          <div className="absolute inset-[19.99%_19.91%_19.99%_20.07%]" data-name="Vector">
            <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 14.4056 14.4056">
              <path d={svgPaths.p106a6b00} fill="var(--fill-0, #4D4E53)" id="Vector" />
            </svg>
          </div>
        </div>
      </div>
      <Body />
      <div className="bg-white content-stretch flex h-[72px] items-center justify-end max-w-[500px] min-w-[500px] p-[16px] relative rounded-bl-[8px] rounded-br-[8px] shrink-0 w-[500px]" data-name="Modal Elemets">
        <div aria-hidden="true" className="absolute border-[#eaeaea] border-solid border-t inset-0 pointer-events-none rounded-bl-[8px] rounded-br-[8px]" />
        <div className="content-stretch flex gap-[8px] h-[40px] items-center justify-end relative shrink-0" data-name="Footer Right Block">
          <div className="bg-[#003c71] content-stretch flex items-center justify-center px-[16px] py-[8px] relative rounded-[8px] shrink-0" data-name="OButton">
            <p className="font-['Inter:Bold',sans-serif] font-bold leading-[24px] not-italic relative shrink-0 text-[16px] text-white whitespace-nowrap">Save</p>
          </div>
        </div>
      </div>
    </div>
  );
}