import svgPaths from "./svg-afm4tb05ut";

function Frame8() {
  return (
    <div className="content-stretch flex flex-col items-start relative shrink-0">
      <p className="font-['Inter:Bold',sans-serif] font-bold leading-[24px] min-w-full not-italic relative shrink-0 text-[#4d4e53] text-[16px] w-[min-content]">Customize Columns</p>
    </div>
  );
}

function LeftSection() {
  return (
    <div className="content-stretch flex gap-[8px] items-center relative shrink-0" data-name="Left Section">
      <Frame8 />
    </div>
  );
}

function Frame6() {
  return (
    <div className="content-stretch flex flex-col gap-[4px] items-start not-italic relative shrink-0 w-full">
      <div className="flex flex-col font-['Inter:Bold',sans-serif] font-bold justify-center leading-[0] relative shrink-0 text-[#202020] text-[16px] w-full">
        <p className="leading-[24px]">Default tab</p>
      </div>
      <p className="font-['Inter:Regular',sans-serif] font-normal leading-[20px] relative shrink-0 text-[#656565] text-[14px] w-full">Choose which tab opens first when you view this page.</p>
    </div>
  );
}

function Radio() {
  return (
    <div className="relative shrink-0 size-[24px]" data-name="Radio">
      <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 24 24">
        <g id="Radio">
          <path d={svgPaths.p1b8494b0} fill="var(--fill-0, #003C71)" id="Vector" />
        </g>
      </svg>
    </div>
  );
}

function Frame() {
  return (
    <div className="content-stretch flex flex-col gap-[4px] items-start justify-center relative shrink-0">
      <p className="font-['Inter:Regular',sans-serif] font-normal leading-[24px] not-italic relative shrink-0 text-[#202020] text-[16px] whitespace-nowrap">Overview</p>
    </div>
  );
}

function List() {
  return (
    <div className="content-stretch flex items-center justify-between py-[8px] relative shrink-0 w-full" data-name="List">
      <div className="content-stretch flex gap-[4px] items-start overflow-clip relative shrink-0" data-name="Radio">
        <Radio />
        <Frame />
      </div>
    </div>
  );
}

function Radio1() {
  return (
    <div className="relative shrink-0 size-[24px]" data-name="Radio">
      <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 24 24">
        <g id="Radio">
          <path d={svgPaths.pf19a9c0} fill="var(--fill-0, #003C71)" id="Vector" />
        </g>
      </svg>
    </div>
  );
}

function Frame1() {
  return (
    <div className="content-stretch flex flex-col gap-[4px] items-start justify-center relative shrink-0">
      <p className="font-['Inter:Regular',sans-serif] font-normal leading-[24px] not-italic relative shrink-0 text-[#202020] text-[16px] whitespace-nowrap">Transaction</p>
    </div>
  );
}

function List1() {
  return (
    <div className="content-stretch flex items-center justify-between py-[8px] relative shrink-0 w-full" data-name="List">
      <div className="content-stretch flex gap-[4px] items-start overflow-clip relative shrink-0" data-name="Radio">
        <Radio1 />
        <Frame1 />
      </div>
    </div>
  );
}

function Frame3() {
  return (
    <div className="content-stretch flex flex-col items-start relative shrink-0 w-full">
      <List />
      <List1 />
    </div>
  );
}

function Frame4() {
  return (
    <div className="content-stretch flex flex-col gap-[8px] items-start relative shrink-0 w-full">
      <Frame6 />
      <Frame3 />
    </div>
  );
}

function Frame7() {
  return (
    <div className="content-stretch flex flex-col gap-[4px] items-start not-italic relative shrink-0 w-full">
      <div className="flex flex-col font-['Inter:Bold',sans-serif] font-bold justify-center leading-[0] relative shrink-0 text-[#202020] text-[16px] w-full">
        <p className="leading-[24px]">Customize Columns</p>
      </div>
      <p className="font-['Inter:Regular',sans-serif] font-normal leading-[20px] relative shrink-0 text-[#656565] text-[14px] w-full">You can control which columns appear in the table and hide the rest by clicking the eye icon. Rearrange the column order by dragging and dropping. The columns at the top will appear first in the data table</p>
    </div>
  );
}

function N() {
  return (
    <div className="absolute inset-[21.88%_17.71%_13.54%_14.99%]" data-name="n">
      <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16.1518 15.5">
        <g id="n">
          <path d={svgPaths.p2b77ab40} fill="var(--fill-0, black)" id="Vector" />
        </g>
      </svg>
    </div>
  );
}

function Hand() {
  return (
    <div className="absolute left-[116px] overflow-clip size-[24px] top-[12px]" data-name="hand 1">
      <N />
    </div>
  );
}

function Label() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-start min-h-px min-w-px relative" data-name="Label">
      <p className="font-['Source_Sans_Pro:SemiBold',sans-serif] leading-[24px] min-w-full not-italic relative shrink-0 text-[#161616] text-[16px] tracking-[0.15px] w-[min-content]">Transaction #</p>
      <p className="font-['Source_Sans_Pro:Regular',sans-serif] h-[20px] leading-[20px] not-italic relative shrink-0 text-[#595959] text-[14px] tracking-[0.1px] w-full">Short explanation needed</p>
      <Hand />
    </div>
  );
}

function Buttons() {
  return (
    <div className="content-stretch flex gap-[8px] items-center relative shrink-0" data-name="Buttons">
      <div className="overflow-clip relative shrink-0 size-[32px]" data-name="Icon Button">
        <div className="absolute inset-[18.75%_4.9%]" data-name="Vector">
          <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 28.8667 20">
            <path d={svgPaths.p2f421800} fill="var(--fill-0, #595959)" id="Vector" />
          </svg>
        </div>
      </div>
    </div>
  );
}

function Group() {
  return (
    <div className="absolute inset-[0_33.94%_0_37.5%]">
      <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 5.71149 20">
        <g id="Group 40537">
          <path d={svgPaths.p32ade700} fill="var(--fill-0, #595959)" id="Vector" />
          <path d={svgPaths.p4290000} fill="var(--fill-0, #595959)" id="Vector_2" />
          <path d={svgPaths.pb73b280} fill="var(--fill-0, #595959)" id="Vector_3" />
        </g>
      </svg>
    </div>
  );
}

function Label1() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-start min-h-px min-w-px not-italic relative" data-name="Label">
      <p className="font-['Source_Sans_Pro:SemiBold',sans-serif] leading-[24px] min-w-full relative shrink-0 text-[#161616] text-[16px] tracking-[0.15px] w-[min-content]">Order Source</p>
      <p className="font-['Source_Sans_Pro:Regular',sans-serif] h-[20px] leading-[20px] relative shrink-0 text-[#595959] text-[14px] tracking-[0.1px] w-full">Short explanation needed</p>
    </div>
  );
}

function Buttons1() {
  return (
    <div className="content-stretch flex gap-[8px] items-center relative shrink-0" data-name="Buttons">
      <div className="overflow-clip relative shrink-0 size-[32px]" data-name="Icon Button">
        <div className="absolute inset-[18.75%_4.9%]" data-name="Vector">
          <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 28.8667 20">
            <path d={svgPaths.p2f421800} fill="var(--fill-0, #595959)" id="Vector" />
          </svg>
        </div>
      </div>
    </div>
  );
}

function Group1() {
  return (
    <div className="absolute inset-[0_33.94%_0_37.5%]">
      <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 5.71149 20">
        <g id="Group 40537">
          <path d={svgPaths.p32ade700} fill="var(--fill-0, #595959)" id="Vector" />
          <path d={svgPaths.p4290000} fill="var(--fill-0, #595959)" id="Vector_2" />
          <path d={svgPaths.pb73b280} fill="var(--fill-0, #595959)" id="Vector_3" />
        </g>
      </svg>
    </div>
  );
}

function Label2() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-start min-h-px min-w-px not-italic relative" data-name="Label">
      <p className="font-['Source_Sans_Pro:SemiBold',sans-serif] leading-[24px] min-w-full relative shrink-0 text-[#161616] text-[16px] tracking-[0.15px] w-[min-content]">Status</p>
      <p className="font-['Source_Sans_Pro:Regular',sans-serif] h-[20px] leading-[20px] relative shrink-0 text-[#595959] text-[14px] tracking-[0.1px] w-full">Short explanation needed</p>
    </div>
  );
}

function Buttons2() {
  return (
    <div className="content-stretch flex gap-[8px] items-center relative shrink-0" data-name="Buttons">
      <div className="overflow-clip relative shrink-0 size-[32px]" data-name="Icon Button">
        <div className="absolute inset-[18.75%_4.9%]" data-name="Vector">
          <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 28.8667 20">
            <path d={svgPaths.p2f421800} fill="var(--fill-0, #595959)" id="Vector" />
          </svg>
        </div>
      </div>
    </div>
  );
}

function Group2() {
  return (
    <div className="absolute inset-[0_33.94%_0_37.5%]">
      <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 5.71149 20">
        <g id="Group 40537">
          <path d={svgPaths.p32ade700} fill="var(--fill-0, #595959)" id="Vector" />
          <path d={svgPaths.p4290000} fill="var(--fill-0, #595959)" id="Vector_2" />
          <path d={svgPaths.pb73b280} fill="var(--fill-0, #595959)" id="Vector_3" />
        </g>
      </svg>
    </div>
  );
}

function Label3() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-start min-h-px min-w-px not-italic relative" data-name="Label">
      <p className="font-['Source_Sans_Pro:SemiBold',sans-serif] leading-[24px] min-w-full relative shrink-0 text-[#161616] text-[16px] tracking-[0.15px] w-[min-content]">Payment</p>
      <p className="font-['Source_Sans_Pro:Regular',sans-serif] h-[20px] leading-[20px] relative shrink-0 text-[#595959] text-[14px] tracking-[0.1px] w-full">Short explanation needed</p>
    </div>
  );
}

function Buttons3() {
  return (
    <div className="content-stretch flex gap-[8px] items-center relative shrink-0" data-name="Buttons">
      <div className="overflow-clip relative shrink-0 size-[32px]" data-name="Icon Button">
        <div className="absolute inset-[18.75%_4.9%]" data-name="Vector">
          <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 28.8667 20">
            <path d={svgPaths.p2f421800} fill="var(--fill-0, #595959)" id="Vector" />
          </svg>
        </div>
      </div>
    </div>
  );
}

function Group3() {
  return (
    <div className="absolute inset-[0_33.94%_0_37.5%]">
      <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 5.71149 20">
        <g id="Group 40537">
          <path d={svgPaths.p32ade700} fill="var(--fill-0, #595959)" id="Vector" />
          <path d={svgPaths.p4290000} fill="var(--fill-0, #595959)" id="Vector_2" />
          <path d={svgPaths.pb73b280} fill="var(--fill-0, #595959)" id="Vector_3" />
        </g>
      </svg>
    </div>
  );
}

function Label4() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-start min-h-px min-w-px not-italic relative" data-name="Label">
      <p className="font-['Source_Sans_Pro:SemiBold',sans-serif] leading-[24px] min-w-full relative shrink-0 text-[#161616] text-[16px] tracking-[0.15px] w-[min-content]">{`Vehicle & Cart Name`}</p>
      <p className="font-['Source_Sans_Pro:Regular',sans-serif] h-[20px] leading-[20px] relative shrink-0 text-[#595959] text-[14px] tracking-[0.1px] w-full">Short explanation needed</p>
    </div>
  );
}

function Buttons4() {
  return (
    <div className="content-stretch flex gap-[8px] items-center relative shrink-0" data-name="Buttons">
      <div className="overflow-clip relative shrink-0 size-[32px]" data-name="Icon Button">
        <div className="absolute inset-[18.75%_4.9%]" data-name="Vector">
          <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 28.8667 20">
            <path d={svgPaths.p2f421800} fill="var(--fill-0, #595959)" id="Vector" />
          </svg>
        </div>
      </div>
    </div>
  );
}

function Group4() {
  return (
    <div className="absolute inset-[0_33.94%_0_37.5%]">
      <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 5.71149 20">
        <g id="Group 40537">
          <path d={svgPaths.p32ade700} fill="var(--fill-0, #595959)" id="Vector" />
          <path d={svgPaths.p4290000} fill="var(--fill-0, #595959)" id="Vector_2" />
          <path d={svgPaths.pb73b280} fill="var(--fill-0, #595959)" id="Vector_3" />
        </g>
      </svg>
    </div>
  );
}

function Label5() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-start min-h-px min-w-px not-italic relative" data-name="Label">
      <p className="font-['Source_Sans_Pro:SemiBold',sans-serif] leading-[24px] min-w-full relative shrink-0 text-[#161616] text-[16px] tracking-[0.15px] w-[min-content]">Salesperson</p>
      <p className="font-['Source_Sans_Pro:Regular',sans-serif] h-[20px] leading-[20px] relative shrink-0 text-[#595959] text-[14px] tracking-[0.1px] w-full">Short explanation needed</p>
    </div>
  );
}

function Buttons5() {
  return (
    <div className="content-stretch flex gap-[8px] items-center relative shrink-0" data-name="Buttons">
      <div className="overflow-clip relative shrink-0 size-[32px]" data-name="Icon Button">
        <div className="absolute inset-[18.75%_4.9%]" data-name="Vector">
          <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 28.8667 20">
            <path d={svgPaths.p2f421800} fill="var(--fill-0, #595959)" id="Vector" />
          </svg>
        </div>
      </div>
    </div>
  );
}

function Group5() {
  return (
    <div className="absolute inset-[0_33.94%_0_37.5%]">
      <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 5.71149 20">
        <g id="Group 40537">
          <path d={svgPaths.p32ade700} fill="var(--fill-0, #595959)" id="Vector" />
          <path d={svgPaths.p4290000} fill="var(--fill-0, #595959)" id="Vector_2" />
          <path d={svgPaths.pb73b280} fill="var(--fill-0, #595959)" id="Vector_3" />
        </g>
      </svg>
    </div>
  );
}

function Label6() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-start min-h-px min-w-px not-italic relative" data-name="Label">
      <p className="font-['Source_Sans_Pro:SemiBold',sans-serif] leading-[24px] min-w-full relative shrink-0 text-[#161616] text-[16px] tracking-[0.15px] w-[min-content]">{`Photos & Notes`}</p>
      <p className="font-['Source_Sans_Pro:Regular',sans-serif] h-[20px] leading-[20px] relative shrink-0 text-[#595959] text-[14px] tracking-[0.1px] w-full">Short explanation needed</p>
    </div>
  );
}

function Buttons6() {
  return (
    <div className="content-stretch flex gap-[8px] items-center relative shrink-0" data-name="Buttons">
      <div className="overflow-clip relative shrink-0 size-[32px]" data-name="Icon Button">
        <div className="absolute inset-[18.75%_4.9%]" data-name="Vector">
          <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 28.8667 20">
            <path d={svgPaths.p2f421800} fill="var(--fill-0, #595959)" id="Vector" />
          </svg>
        </div>
      </div>
    </div>
  );
}

function Group6() {
  return (
    <div className="absolute inset-[0_33.94%_0_37.5%]">
      <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 5.71149 20">
        <g id="Group 40537">
          <path d={svgPaths.p32ade700} fill="var(--fill-0, #595959)" id="Vector" />
          <path d={svgPaths.p4290000} fill="var(--fill-0, #595959)" id="Vector_2" />
          <path d={svgPaths.pb73b280} fill="var(--fill-0, #595959)" id="Vector_3" />
        </g>
      </svg>
    </div>
  );
}

function Label7() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-start min-h-px min-w-px not-italic relative" data-name="Label">
      <p className="font-['Source_Sans_Pro:SemiBold',sans-serif] leading-[24px] min-w-full relative shrink-0 text-[#161616] text-[16px] tracking-[0.15px] w-[min-content]">Last Memo</p>
      <p className="font-['Source_Sans_Pro:Regular',sans-serif] h-[20px] leading-[20px] relative shrink-0 text-[#595959] text-[14px] tracking-[0.1px] w-full">Short explanation needed</p>
    </div>
  );
}

function Buttons7() {
  return (
    <div className="content-stretch flex gap-[8px] items-center relative shrink-0" data-name="Buttons">
      <div className="overflow-clip relative shrink-0 size-[32px]" data-name="Icon Button">
        <div className="absolute inset-[18.75%_4.9%]" data-name="Vector">
          <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 28.8667 20">
            <path d={svgPaths.p2f421800} fill="var(--fill-0, #595959)" id="Vector" />
          </svg>
        </div>
      </div>
    </div>
  );
}

function Group7() {
  return (
    <div className="absolute inset-[0_33.94%_0_37.5%]">
      <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 5.71149 20">
        <g id="Group 40537">
          <path d={svgPaths.p32ade700} fill="var(--fill-0, #595959)" id="Vector" />
          <path d={svgPaths.p4290000} fill="var(--fill-0, #595959)" id="Vector_2" />
          <path d={svgPaths.pb73b280} fill="var(--fill-0, #595959)" id="Vector_3" />
        </g>
      </svg>
    </div>
  );
}

function Label8() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-start min-h-px min-w-px not-italic relative" data-name="Label">
      <p className="font-['Source_Sans_Pro:SemiBold',sans-serif] leading-[24px] min-w-full relative shrink-0 text-[#161616] text-[16px] tracking-[0.15px] w-[min-content]">Control</p>
      <p className="font-['Source_Sans_Pro:Regular',sans-serif] h-[20px] leading-[20px] relative shrink-0 text-[#595959] text-[14px] tracking-[0.1px] w-full">Short explanation needed</p>
    </div>
  );
}

function Buttons8() {
  return (
    <div className="content-stretch flex gap-[8px] items-center relative shrink-0" data-name="Buttons">
      <div className="overflow-clip relative shrink-0 size-[32px]" data-name="Icon Button">
        <div className="absolute inset-[10.52%_4.9%]" data-name="Vector">
          <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 28.8667 25.2667">
            <path d={svgPaths.p83448f0} fill="var(--fill-0, #595959)" id="Vector" />
          </svg>
        </div>
      </div>
    </div>
  );
}

function Group8() {
  return (
    <div className="absolute inset-[0_33.94%_0_37.5%]">
      <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 5.71149 20">
        <g id="Group 40537">
          <path d={svgPaths.p32ade700} fill="var(--fill-0, #595959)" id="Vector" />
          <path d={svgPaths.p4290000} fill="var(--fill-0, #595959)" id="Vector_2" />
          <path d={svgPaths.pb73b280} fill="var(--fill-0, #595959)" id="Vector_3" />
        </g>
      </svg>
    </div>
  );
}

function Label9() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-start min-h-px min-w-px not-italic relative" data-name="Label">
      <p className="font-['Source_Sans_Pro:SemiBold',sans-serif] leading-[24px] min-w-full relative shrink-0 text-[#161616] text-[16px] tracking-[0.15px] w-[min-content]">Submitted By</p>
      <p className="font-['Source_Sans_Pro:Regular',sans-serif] h-[20px] leading-[20px] relative shrink-0 text-[#595959] text-[14px] tracking-[0.1px] w-full">Short explanation needed</p>
    </div>
  );
}

function Buttons9() {
  return (
    <div className="content-stretch flex gap-[8px] items-center relative shrink-0" data-name="Buttons">
      <div className="overflow-clip relative shrink-0 size-[32px]" data-name="Icon Button">
        <div className="absolute inset-[10.52%_4.9%]" data-name="Vector">
          <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 28.8667 25.2667">
            <path d={svgPaths.p83448f0} fill="var(--fill-0, #595959)" id="Vector" />
          </svg>
        </div>
      </div>
    </div>
  );
}

function Group9() {
  return (
    <div className="absolute inset-[0_33.94%_0_37.5%]">
      <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 5.71149 20">
        <g id="Group 40537">
          <path d={svgPaths.p32ade700} fill="var(--fill-0, #595959)" id="Vector" />
          <path d={svgPaths.p4290000} fill="var(--fill-0, #595959)" id="Vector_2" />
          <path d={svgPaths.pb73b280} fill="var(--fill-0, #595959)" id="Vector_3" />
        </g>
      </svg>
    </div>
  );
}

function Frame2() {
  return (
    <div className="content-stretch flex flex-col gap-[8px] items-start relative shrink-0 w-[279px]">
      <div className="bg-white relative rounded-[8px] shadow-[0px_-0.5px_1px_0px_rgba(0,0,0,0.15),0px_2px_4px_-1px_rgba(0,0,0,0.2),0px_4px_5px_0px_rgba(0,0,0,0.14),0px_1px_10px_0px_rgba(0,0,0,0.12)] shrink-0 w-full" data-name="Draggable card">
        <div className="flex flex-row items-center overflow-clip rounded-[inherit] size-full">
          <div className="content-stretch flex gap-[10px] items-center pl-[24px] pr-[16px] py-[8px] relative w-full">
            <Label />
            <Buttons />
            <div className="-translate-y-1/2 absolute left-0 overflow-clip size-[20px] top-[calc(50%-1px)]" data-name="Drag">
              <Group />
            </div>
          </div>
        </div>
      </div>
      <div className="bg-white relative rounded-[8px] shadow-[0px_-0.5px_1px_0px_rgba(0,0,0,0.15),0px_2px_4px_-1px_rgba(0,0,0,0.2),0px_4px_5px_0px_rgba(0,0,0,0.14),0px_1px_10px_0px_rgba(0,0,0,0.12)] shrink-0 w-full" data-name="Draggable card">
        <div className="flex flex-row items-center overflow-clip rounded-[inherit] size-full">
          <div className="content-stretch flex gap-[10px] items-center pl-[24px] pr-[16px] py-[8px] relative w-full">
            <Label1 />
            <Buttons1 />
            <div className="-translate-y-1/2 absolute left-0 overflow-clip size-[20px] top-[calc(50%-1px)]" data-name="Drag">
              <Group1 />
            </div>
          </div>
        </div>
      </div>
      <div className="bg-white relative rounded-[8px] shadow-[0px_-0.5px_1px_0px_rgba(0,0,0,0.15),0px_2px_4px_-1px_rgba(0,0,0,0.2),0px_4px_5px_0px_rgba(0,0,0,0.14),0px_1px_10px_0px_rgba(0,0,0,0.12)] shrink-0 w-full" data-name="Draggable card">
        <div className="flex flex-row items-center overflow-clip rounded-[inherit] size-full">
          <div className="content-stretch flex gap-[10px] items-center pl-[24px] pr-[16px] py-[8px] relative w-full">
            <Label2 />
            <Buttons2 />
            <div className="-translate-y-1/2 absolute left-0 overflow-clip size-[20px] top-[calc(50%-1px)]" data-name="Drag">
              <Group2 />
            </div>
          </div>
        </div>
      </div>
      <div className="bg-white relative rounded-[8px] shadow-[0px_-0.5px_1px_0px_rgba(0,0,0,0.15),0px_2px_4px_-1px_rgba(0,0,0,0.2),0px_4px_5px_0px_rgba(0,0,0,0.14),0px_1px_10px_0px_rgba(0,0,0,0.12)] shrink-0 w-full" data-name="Draggable card">
        <div className="flex flex-row items-center overflow-clip rounded-[inherit] size-full">
          <div className="content-stretch flex gap-[10px] items-center pl-[24px] pr-[16px] py-[8px] relative w-full">
            <Label3 />
            <Buttons3 />
            <div className="-translate-y-1/2 absolute left-0 overflow-clip size-[20px] top-[calc(50%-1px)]" data-name="Drag">
              <Group3 />
            </div>
          </div>
        </div>
      </div>
      <div className="bg-white relative rounded-[8px] shadow-[0px_-0.5px_1px_0px_rgba(0,0,0,0.15),0px_2px_4px_-1px_rgba(0,0,0,0.2),0px_4px_5px_0px_rgba(0,0,0,0.14),0px_1px_10px_0px_rgba(0,0,0,0.12)] shrink-0 w-full" data-name="Draggable card">
        <div className="flex flex-row items-center overflow-clip rounded-[inherit] size-full">
          <div className="content-stretch flex gap-[10px] items-center pl-[24px] pr-[16px] py-[8px] relative w-full">
            <Label4 />
            <Buttons4 />
            <div className="-translate-y-1/2 absolute left-0 overflow-clip size-[20px] top-[calc(50%-1px)]" data-name="Drag">
              <Group4 />
            </div>
          </div>
        </div>
      </div>
      <div className="bg-white relative rounded-[8px] shadow-[0px_-0.5px_1px_0px_rgba(0,0,0,0.15),0px_2px_4px_-1px_rgba(0,0,0,0.2),0px_4px_5px_0px_rgba(0,0,0,0.14),0px_1px_10px_0px_rgba(0,0,0,0.12)] shrink-0 w-full" data-name="Draggable card">
        <div className="flex flex-row items-center overflow-clip rounded-[inherit] size-full">
          <div className="content-stretch flex gap-[10px] items-center pl-[24px] pr-[16px] py-[8px] relative w-full">
            <Label5 />
            <Buttons5 />
            <div className="-translate-y-1/2 absolute left-0 overflow-clip size-[20px] top-[calc(50%-1px)]" data-name="Drag">
              <Group5 />
            </div>
          </div>
        </div>
      </div>
      <div className="bg-white relative rounded-[8px] shadow-[0px_-0.5px_1px_0px_rgba(0,0,0,0.15),0px_2px_4px_-1px_rgba(0,0,0,0.2),0px_4px_5px_0px_rgba(0,0,0,0.14),0px_1px_10px_0px_rgba(0,0,0,0.12)] shrink-0 w-full" data-name="Draggable card">
        <div className="flex flex-row items-center overflow-clip rounded-[inherit] size-full">
          <div className="content-stretch flex gap-[10px] items-center pl-[24px] pr-[16px] py-[8px] relative w-full">
            <Label6 />
            <Buttons6 />
            <div className="-translate-y-1/2 absolute left-0 overflow-clip size-[20px] top-[calc(50%-1px)]" data-name="Drag">
              <Group6 />
            </div>
          </div>
        </div>
      </div>
      <div className="bg-white relative rounded-[8px] shadow-[0px_-0.5px_1px_0px_rgba(0,0,0,0.15),0px_2px_4px_-1px_rgba(0,0,0,0.2),0px_4px_5px_0px_rgba(0,0,0,0.14),0px_1px_10px_0px_rgba(0,0,0,0.12)] shrink-0 w-full" data-name="Draggable card">
        <div className="flex flex-row items-center overflow-clip rounded-[inherit] size-full">
          <div className="content-stretch flex gap-[10px] items-center pl-[24px] pr-[16px] py-[8px] relative w-full">
            <Label7 />
            <Buttons7 />
            <div className="-translate-y-1/2 absolute left-0 overflow-clip size-[20px] top-[calc(50%-1px)]" data-name="Drag">
              <Group7 />
            </div>
          </div>
        </div>
      </div>
      <div className="bg-white relative rounded-[8px] shadow-[0px_-0.5px_1px_0px_rgba(0,0,0,0.15),0px_2px_4px_-1px_rgba(0,0,0,0.2),0px_4px_5px_0px_rgba(0,0,0,0.14),0px_1px_10px_0px_rgba(0,0,0,0.12)] shrink-0 w-full" data-name="Draggable card">
        <div className="flex flex-row items-center overflow-clip rounded-[inherit] size-full">
          <div className="content-stretch flex gap-[10px] items-center pl-[24px] pr-[16px] py-[8px] relative w-full">
            <Label8 />
            <Buttons8 />
            <div className="-translate-y-1/2 absolute left-0 overflow-clip size-[20px] top-[calc(50%-1px)]" data-name="Drag">
              <Group8 />
            </div>
          </div>
        </div>
      </div>
      <div className="bg-white relative rounded-[8px] shadow-[0px_-0.5px_1px_0px_rgba(0,0,0,0.15),0px_2px_4px_-1px_rgba(0,0,0,0.2),0px_4px_5px_0px_rgba(0,0,0,0.14),0px_1px_10px_0px_rgba(0,0,0,0.12)] shrink-0 w-full" data-name="Draggable card">
        <div className="flex flex-row items-center overflow-clip rounded-[inherit] size-full">
          <div className="content-stretch flex gap-[10px] items-center pl-[24px] pr-[16px] py-[8px] relative w-full">
            <Label9 />
            <Buttons9 />
            <div className="-translate-y-1/2 absolute left-0 overflow-clip size-[20px] top-[calc(50%-1px)]" data-name="Drag">
              <Group9 />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function Frame5() {
  return (
    <div className="content-stretch flex flex-col gap-[16px] items-start relative shrink-0 w-full">
      <Frame7 />
      <Frame2 />
    </div>
  );
}

function Body() {
  return (
    <div className="bg-white relative shrink-0 w-full" data-name="Body">
      <div className="overflow-clip rounded-[inherit] size-full">
        <div className="content-stretch flex flex-col gap-[24px] items-start pb-[16px] pt-[24px] px-[16px] relative w-full">
          <Frame4 />
          <Frame5 />
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
            <p className="font-['Inter:Bold',sans-serif] font-bold leading-[24px] not-italic relative shrink-0 text-[16px] text-white whitespace-nowrap">Save</p>
          </div>
        </div>
      </div>
    </div>
  );
}