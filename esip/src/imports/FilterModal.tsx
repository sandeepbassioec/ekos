import svgPaths from "./svg-ze22aoktid";

function Frame6() {
  return (
    <div className="content-stretch flex flex-col items-start relative shrink-0">
      <p className="font-['Inter:Bold',sans-serif] font-bold leading-[24px] min-w-full not-italic relative shrink-0 text-[#4d4e53] text-[16px] w-[min-content]">Filters</p>
    </div>
  );
}

function LeftSection() {
  return (
    <div className="content-stretch flex gap-[8px] items-center relative shrink-0" data-name="Left Section">
      <Frame6 />
    </div>
  );
}

function Body1() {
  return (
    <div className="bg-white content-stretch flex gap-[2px] items-center justify-center px-[4px] relative rounded-[4px] shrink-0" data-name="Body">
      <p className="font-['Inter:Regular',sans-serif] font-normal leading-[16px] not-italic relative shrink-0 text-[#4d4e53] text-[12px] whitespace-nowrap">Start Date</p>
    </div>
  );
}

function Label() {
  return (
    <div className="absolute content-stretch flex flex-col items-start left-0 px-[8px] top-[-10px] w-[307px] z-[2]" data-name="Label">
      <Body1 />
    </div>
  );
}

function DateInput() {
  return (
    <div className="content-stretch flex gap-[8px] items-center relative shrink-0" data-name="Date Input">
      <div className="overflow-clip relative shrink-0 size-[24px]" data-name="Calendar Icon">
        <div className="absolute inset-[10%_15%_15%_15%]" data-name="Vector">
          <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16.8 18">
            <path d={svgPaths.p2c0d6100} fill="var(--fill-0, #4D4E53)" id="Vector" />
          </svg>
        </div>
      </div>
      <div className="content-stretch flex flex-col items-start justify-center relative shrink-0" data-name="Value">
        <p className="font-['Inter:Regular',sans-serif] font-normal leading-[24px] not-italic relative shrink-0 text-[#4d4e53] text-[16px] whitespace-nowrap">DD/MM/YYYY</p>
      </div>
    </div>
  );
}

function NumberField() {
  return (
    <div className="bg-white mb-[-6px] relative rounded-[8px] shrink-0 w-full z-[1]" data-name="Number Field">
      <div aria-hidden="true" className="absolute border border-[#bfbfbf] border-solid inset-0 pointer-events-none rounded-[8px]" />
      <div className="flex flex-row items-center size-full">
        <div className="content-stretch flex gap-[10px] items-center px-[16px] py-[8px] relative w-full">
          <DateInput />
        </div>
      </div>
    </div>
  );
}

function LabelContainer() {
  return (
    <div className="content-stretch flex flex-col isolate items-start pb-[6px] relative shrink-0 w-full" data-name="Label Container">
      <Label />
      <NumberField />
    </div>
  );
}

function Body2() {
  return (
    <div className="bg-white content-stretch flex gap-[2px] items-center justify-center px-[4px] relative rounded-[4px] shrink-0" data-name="Body">
      <p className="font-['Inter:Regular',sans-serif] font-normal leading-[16px] not-italic relative shrink-0 text-[#4d4e53] text-[12px] whitespace-nowrap">End Date</p>
    </div>
  );
}

function Label1() {
  return (
    <div className="absolute content-stretch flex flex-col items-start left-0 px-[8px] top-[-10px] w-[307px] z-[2]" data-name="Label">
      <Body2 />
    </div>
  );
}

function DateInput1() {
  return (
    <div className="content-stretch flex gap-[8px] items-center relative shrink-0" data-name="Date Input">
      <div className="overflow-clip relative shrink-0 size-[24px]" data-name="Calendar Icon">
        <div className="absolute inset-[10%_15%_15%_15%]" data-name="Vector">
          <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16.8 18">
            <path d={svgPaths.p2c0d6100} fill="var(--fill-0, #4D4E53)" id="Vector" />
          </svg>
        </div>
      </div>
      <div className="content-stretch flex flex-col items-start justify-center relative shrink-0" data-name="Value">
        <p className="font-['Inter:Regular',sans-serif] font-normal leading-[24px] not-italic relative shrink-0 text-[#4d4e53] text-[16px] whitespace-nowrap">DD/MM/YYYY</p>
      </div>
    </div>
  );
}

function NumberField1() {
  return (
    <div className="bg-white mb-[-6px] relative rounded-[8px] shrink-0 w-full z-[1]" data-name="Number Field">
      <div aria-hidden="true" className="absolute border border-[#bfbfbf] border-solid inset-0 pointer-events-none rounded-[8px]" />
      <div className="flex flex-row items-center size-full">
        <div className="content-stretch flex gap-[10px] items-center px-[16px] py-[8px] relative w-full">
          <DateInput1 />
        </div>
      </div>
    </div>
  );
}

function LabelContainer1() {
  return (
    <div className="content-stretch flex flex-col isolate items-start pb-[6px] relative shrink-0 w-full" data-name="Label Container">
      <Label1 />
      <NumberField1 />
    </div>
  );
}

function Frame() {
  return (
    <div className="content-stretch flex gap-[16px] items-center relative shrink-0">
      <div className="content-stretch flex flex-col gap-[4px] items-start relative shrink-0 w-[222px]" data-name="Number Field">
        <LabelContainer />
      </div>
      <div className="content-stretch flex flex-col gap-[4px] items-start relative shrink-0 w-[222px]" data-name="Number Field">
        <LabelContainer1 />
      </div>
    </div>
  );
}

function Frame3() {
  return (
    <div className="content-stretch flex flex-col gap-[20px] items-start relative shrink-0 w-full">
      <div className="flex flex-col font-['Inter:Bold',sans-serif] font-bold justify-center leading-[0] min-w-full not-italic relative shrink-0 text-[#202020] text-[16px] w-[min-content]">
        <p className="leading-[24px]">Date Range</p>
      </div>
      <Frame />
    </div>
  );
}

function CheckboxV() {
  return (
    <div className="relative shrink-0 size-[24px]" data-name="Checkbox V2">
      <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 24 24">
        <g id="Checkbox V2">
          <path d={svgPaths.p3640ce00} fill="var(--fill-0, #003C71)" id="Vector" />
        </g>
      </svg>
    </div>
  );
}

function LabelHeading() {
  return (
    <div className="content-stretch flex gap-[4px] items-start relative shrink-0" data-name="Label heading">
      <p className="font-['Inter:Regular',sans-serif] font-normal leading-[24px] not-italic relative shrink-0 text-[#202020] text-[16px] whitespace-nowrap">Lexus</p>
    </div>
  );
}

function Label2() {
  return (
    <div className="content-stretch flex flex-col gap-[4px] items-start justify-center relative shrink-0" data-name="Label">
      <LabelHeading />
    </div>
  );
}

function List() {
  return (
    <div className="content-stretch flex items-center justify-between py-[8px] relative shrink-0 w-full" data-name="List">
      <div className="content-stretch flex gap-[4px] items-start overflow-clip relative rounded-[2px] shrink-0" data-name="Edit Icon">
        <CheckboxV />
        <Label2 />
      </div>
      <p className="font-['Inter:Bold',sans-serif] font-bold leading-[24px] not-italic relative shrink-0 text-[#4d4e53] text-[16px] whitespace-nowrap">2</p>
    </div>
  );
}

function CheckboxV1() {
  return (
    <div className="relative shrink-0 size-[24px]" data-name="Checkbox V2">
      <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 24 24">
        <g id="Checkbox V2">
          <path d={svgPaths.p3640ce00} fill="var(--fill-0, #003C71)" id="Vector" />
        </g>
      </svg>
    </div>
  );
}

function LabelHeading1() {
  return (
    <div className="content-stretch flex gap-[4px] items-start relative shrink-0" data-name="Label heading">
      <p className="font-['Inter:Regular',sans-serif] font-normal leading-[24px] not-italic relative shrink-0 text-[#202020] text-[16px] whitespace-nowrap">Toyota</p>
    </div>
  );
}

function Label3() {
  return (
    <div className="content-stretch flex flex-col gap-[4px] items-start justify-center relative shrink-0" data-name="Label">
      <LabelHeading1 />
    </div>
  );
}

function List1() {
  return (
    <div className="content-stretch flex items-center justify-between py-[8px] relative shrink-0 w-full" data-name="List">
      <div className="content-stretch flex gap-[4px] items-start overflow-clip relative rounded-[2px] shrink-0" data-name="Edit Icon">
        <CheckboxV1 />
        <Label3 />
      </div>
      <p className="font-['Inter:Bold',sans-serif] font-bold leading-[24px] not-italic relative shrink-0 text-[#4d4e53] text-[16px] whitespace-nowrap">2</p>
    </div>
  );
}

function CheckboxV2() {
  return (
    <div className="relative shrink-0 size-[24px]" data-name="Checkbox V2">
      <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 24 24">
        <g id="Checkbox V2">
          <path d={svgPaths.p3640ce00} fill="var(--fill-0, #003C71)" id="Vector" />
        </g>
      </svg>
    </div>
  );
}

function LabelHeading2() {
  return (
    <div className="content-stretch flex gap-[4px] items-start relative shrink-0" data-name="Label heading">
      <p className="font-['Inter:Regular',sans-serif] font-normal leading-[24px] not-italic relative shrink-0 text-[#202020] text-[16px] whitespace-nowrap">Mazda</p>
    </div>
  );
}

function Label4() {
  return (
    <div className="content-stretch flex flex-col gap-[4px] items-start justify-center relative shrink-0" data-name="Label">
      <LabelHeading2 />
    </div>
  );
}

function List2() {
  return (
    <div className="content-stretch flex items-center justify-between py-[8px] relative shrink-0 w-full" data-name="List">
      <div className="content-stretch flex gap-[4px] items-start overflow-clip relative rounded-[2px] shrink-0" data-name="Edit Icon">
        <CheckboxV2 />
        <Label4 />
      </div>
      <p className="font-['Inter:Bold',sans-serif] font-bold leading-[24px] not-italic relative shrink-0 text-[#4d4e53] text-[16px] whitespace-nowrap">2</p>
    </div>
  );
}

function Frame1() {
  return (
    <div className="content-stretch flex flex-col items-start relative shrink-0 w-full">
      <List />
      <List1 />
      <List2 />
    </div>
  );
}

function Frame4() {
  return (
    <div className="content-stretch flex flex-col gap-[8px] items-start relative shrink-0 w-[267px]">
      <div className="flex flex-col font-['Inter:Bold',sans-serif] font-bold justify-center leading-[0] not-italic relative shrink-0 text-[#202020] text-[16px] w-full">
        <p className="leading-[24px]">OEM</p>
      </div>
      <Frame1 />
    </div>
  );
}

function CheckboxV3() {
  return (
    <div className="relative shrink-0 size-[24px]" data-name="Checkbox V2">
      <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 24 24">
        <g id="Checkbox V2">
          <path d={svgPaths.p3640ce00} fill="var(--fill-0, #003C71)" id="Vector" />
        </g>
      </svg>
    </div>
  );
}

function LabelHeading3() {
  return (
    <div className="content-stretch flex gap-[4px] items-start relative shrink-0" data-name="Label heading">
      <p className="font-['Inter:Regular',sans-serif] font-normal leading-[24px] not-italic relative shrink-0 text-[#202020] text-[16px] whitespace-nowrap">Lexus</p>
    </div>
  );
}

function Label5() {
  return (
    <div className="content-stretch flex flex-col gap-[4px] items-start justify-center relative shrink-0" data-name="Label">
      <LabelHeading3 />
    </div>
  );
}

function List3() {
  return (
    <div className="content-stretch flex items-center justify-between py-[8px] relative shrink-0 w-full" data-name="List">
      <div className="content-stretch flex gap-[4px] items-start overflow-clip relative rounded-[2px] shrink-0" data-name="Edit Icon">
        <CheckboxV3 />
        <Label5 />
      </div>
      <p className="font-['Inter:Bold',sans-serif] font-bold leading-[24px] not-italic relative shrink-0 text-[#4d4e53] text-[16px] whitespace-nowrap">2</p>
    </div>
  );
}

function CheckboxV4() {
  return (
    <div className="relative shrink-0 size-[24px]" data-name="Checkbox V2">
      <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 24 24">
        <g id="Checkbox V2">
          <path d={svgPaths.p3640ce00} fill="var(--fill-0, #003C71)" id="Vector" />
        </g>
      </svg>
    </div>
  );
}

function LabelHeading4() {
  return (
    <div className="content-stretch flex gap-[4px] items-start relative shrink-0" data-name="Label heading">
      <p className="font-['Inter:Regular',sans-serif] font-normal leading-[24px] not-italic relative shrink-0 text-[#202020] text-[16px] whitespace-nowrap">Toyota</p>
    </div>
  );
}

function Label6() {
  return (
    <div className="content-stretch flex flex-col gap-[4px] items-start justify-center relative shrink-0" data-name="Label">
      <LabelHeading4 />
    </div>
  );
}

function List4() {
  return (
    <div className="content-stretch flex items-center justify-between py-[8px] relative shrink-0 w-full" data-name="List">
      <div className="content-stretch flex gap-[4px] items-start overflow-clip relative rounded-[2px] shrink-0" data-name="Edit Icon">
        <CheckboxV4 />
        <Label6 />
      </div>
      <p className="font-['Inter:Bold',sans-serif] font-bold leading-[24px] not-italic relative shrink-0 text-[#4d4e53] text-[16px] whitespace-nowrap">2</p>
    </div>
  );
}

function CheckboxV5() {
  return (
    <div className="relative shrink-0 size-[24px]" data-name="Checkbox V2">
      <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 24 24">
        <g id="Checkbox V2">
          <path d={svgPaths.p3640ce00} fill="var(--fill-0, #003C71)" id="Vector" />
        </g>
      </svg>
    </div>
  );
}

function LabelHeading5() {
  return (
    <div className="content-stretch flex gap-[4px] items-start relative shrink-0" data-name="Label heading">
      <p className="font-['Inter:Regular',sans-serif] font-normal leading-[24px] not-italic relative shrink-0 text-[#202020] text-[16px] whitespace-nowrap">Mazda</p>
    </div>
  );
}

function Label7() {
  return (
    <div className="content-stretch flex flex-col gap-[4px] items-start justify-center relative shrink-0" data-name="Label">
      <LabelHeading5 />
    </div>
  );
}

function List5() {
  return (
    <div className="content-stretch flex items-center justify-between py-[8px] relative shrink-0 w-full" data-name="List">
      <div className="content-stretch flex gap-[4px] items-start overflow-clip relative rounded-[2px] shrink-0" data-name="Edit Icon">
        <CheckboxV5 />
        <Label7 />
      </div>
      <p className="font-['Inter:Bold',sans-serif] font-bold leading-[24px] not-italic relative shrink-0 text-[#4d4e53] text-[16px] whitespace-nowrap">2</p>
    </div>
  );
}

function Frame5() {
  return (
    <div className="content-stretch flex flex-col items-start relative shrink-0 w-full">
      <List3 />
      <List4 />
      <List5 />
    </div>
  );
}

function Frame2() {
  return (
    <div className="content-stretch flex flex-col gap-[8px] items-start relative shrink-0 w-[267px]">
      <div className="flex flex-col font-['Inter:Bold',sans-serif] font-bold justify-center leading-[0] not-italic relative shrink-0 text-[#202020] text-[16px] w-full">
        <p className="leading-[24px]">Salesperson</p>
      </div>
      <Frame5 />
    </div>
  );
}

function Body() {
  return (
    <div className="bg-white relative shrink-0 w-full" data-name="Body">
      <div className="overflow-clip rounded-[inherit] size-full">
        <div className="content-stretch flex flex-col gap-[24px] items-start p-[24px] relative w-full">
          <Frame3 />
          <Frame4 />
          <Frame2 />
        </div>
      </div>
    </div>
  );
}

export default function FilterModal() {
  return (
    <div className="content-stretch flex flex-col items-start relative size-full" data-name="Filter Modal">
      <div className="bg-white content-stretch flex items-center justify-between max-w-[700px] min-w-[700px] px-[16px] py-[12px] relative rounded-tl-[8px] rounded-tr-[8px] shrink-0 w-[700px]" data-name="Modal Elemets">
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
      <div className="bg-white content-stretch flex h-[72px] items-center justify-end max-w-[700px] min-w-[700px] p-[16px] relative rounded-bl-[8px] rounded-br-[8px] shrink-0 w-[700px]" data-name="Modal Elemets">
        <div aria-hidden="true" className="absolute border-[#eaeaea] border-solid border-t inset-0 pointer-events-none rounded-bl-[8px] rounded-br-[8px]" />
        <div className="content-stretch flex gap-[8px] h-[40px] items-center justify-end relative shrink-0" data-name="Footer Right Block">
          <div className="bg-white content-stretch flex items-center justify-center px-[16px] py-[8px] relative rounded-[8px] shrink-0" data-name="OButton">
            <p className="font-['Inter:Bold',sans-serif] font-bold leading-[24px] not-italic relative shrink-0 text-[#003c71] text-[16px] whitespace-nowrap">Cancel</p>
          </div>
          <div className="bg-[#003c71] content-stretch flex items-center justify-center px-[16px] py-[8px] relative rounded-[8px] shrink-0" data-name="OButton">
            <p className="font-['Inter:Bold',sans-serif] font-bold leading-[24px] not-italic relative shrink-0 text-[16px] text-white whitespace-nowrap">Apply</p>
          </div>
        </div>
      </div>
    </div>
  );
}