import svgPaths from "./svg-dw9gm5jhhq";

function Frame1() {
  return (
    <div className="content-stretch flex flex-col items-start relative shrink-0">
      <p className="font-['Inter:Bold',sans-serif] font-bold leading-[24px] min-w-full not-italic relative shrink-0 text-[#4d4e53] text-[16px] w-[min-content]">Vehicle details</p>
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

function Container1() {
  return (
    <div className="content-stretch flex gap-[8px] items-center relative shrink-0 w-full" data-name="Container">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Bold',sans-serif] font-bold justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[14px]">
        <p className="leading-[20px]">Program/Pricing information</p>
      </div>
    </div>
  );
}

function MainTextTrailingIcon() {
  return (
    <div className="content-stretch flex gap-[4px] items-center min-w-[4px] relative shrink-0 w-[151px]" data-name="Main text & trailing icon">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic relative shrink-0 text-[#656565] text-[14px] whitespace-nowrap">
        <p className="leading-[20px]">Program:</p>
      </div>
    </div>
  );
}

function SizeLargeTypeMainTextStateDefault() {
  return (
    <div className="bg-white content-stretch flex gap-[8px] items-start py-[4px] relative shrink-0" data-name="Size=Large, Type=Main text, State=Default">
      <MainTextTrailingIcon />
    </div>
  );
}

function MainTextTrailingIcon1() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center min-h-px min-w-[4px] relative" data-name="Main text & trailing icon">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#161616] text-[0px]">
        <p className="text-[14px]">
          <span className="leading-[20px] text-[#656565]">Current pricing</span>
          <span className="leading-[20px] text-[#656565]">:</span>
          <span className="leading-[20px] text-[#656565]">{` `}</span>
          <span className="leading-[20px] text-[#202020]">TTH (Transit 2015)</span>
        </p>
      </div>
    </div>
  );
}

function SizeLargeTypeMainTextStateDefault1() {
  return (
    <div className="bg-white content-stretch flex gap-[8px] items-start py-[4px] relative shrink-0 w-full" data-name="Size=Large, Type=Main text, State=Default">
      <MainTextTrailingIcon1 />
    </div>
  );
}

function Container2() {
  return (
    <div className="content-stretch flex flex-col items-start relative shrink-0 w-full" data-name="Container">
      <SizeLargeTypeMainTextStateDefault />
      <SizeLargeTypeMainTextStateDefault1 />
    </div>
  );
}

function ProductDetails() {
  return (
    <div className="bg-white flex-[1_0_0] min-h-px min-w-px relative rounded-[8px] self-stretch" data-name="Product Details">
      <div aria-hidden="true" className="absolute border border-[#eaeaea] border-solid inset-0 pointer-events-none rounded-[8px]" />
      <div className="content-stretch flex flex-col gap-[8px] items-start p-[16px] relative size-full">
        <Container1 />
        <Container2 />
      </div>
    </div>
  );
}

function Container3() {
  return (
    <div className="content-stretch flex gap-[16px] items-center relative shrink-0 w-full" data-name="Container">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Bold',sans-serif] font-bold justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[14px]">
        <p className="leading-[20px]">Order information</p>
      </div>
    </div>
  );
}

function MainTextTrailingIcon2() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center min-h-px min-w-[4px] relative" data-name="Main text & trailing icon">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[0px]">
        <p className="text-[14px]">
          <span className="leading-[20px] text-[#656565]">Transaction ID</span>
          <span className="leading-[20px] text-[#656565]">:</span>
          <span className="leading-[20px] text-[#656565]">{` `}</span>
          <span className="leading-[20px]">RLM: 063882980</span>
        </p>
      </div>
    </div>
  );
}

function SizeLargeTypeMainTextStateDefault2() {
  return (
    <div className="bg-white content-stretch flex gap-[8px] items-start py-[4px] relative shrink-0 w-full" data-name="Size=Large, Type=Main text, State=Default">
      <MainTextTrailingIcon2 />
    </div>
  );
}

function MainTextTrailingIcon3() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center min-h-px min-w-[4px] relative" data-name="Main text & trailing icon">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#161616] text-[0px]">
        <p className="text-[#656565] text-[14px]">
          <span className="leading-[20px]">P.O.#</span>
          <span className="leading-[20px]">:</span>
          <span className="leading-[20px]">{` `}</span>
        </p>
      </div>
    </div>
  );
}

function SizeLargeTypeMainTextStateDefault3() {
  return (
    <div className="bg-white content-stretch flex gap-[8px] items-start py-[4px] relative shrink-0 w-full" data-name="Size=Large, Type=Main text, State=Default">
      <MainTextTrailingIcon3 />
    </div>
  );
}

function MainTextTrailingIcon4() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center min-h-px min-w-[4px] relative" data-name="Main text & trailing icon">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[0px]">
        <p className="text-[14px]">
          <span className="leading-[20px] text-[#656565]">Payment ID</span>
          <span className="leading-[20px] text-[#656565]">:</span>
          <span className="leading-[20px] text-[#656565]">{` `}</span>
          <span className="leading-[20px]">0</span>
        </p>
      </div>
    </div>
  );
}

function SizeLargeTypeMainTextStateDefault4() {
  return (
    <div className="bg-white content-stretch flex gap-[8px] items-start py-[4px] relative shrink-0 w-full" data-name="Size=Large, Type=Main text, State=Default">
      <MainTextTrailingIcon4 />
    </div>
  );
}

function Container4() {
  return (
    <div className="content-stretch flex flex-col items-start relative shrink-0 w-full" data-name="Container">
      <SizeLargeTypeMainTextStateDefault2 />
      <SizeLargeTypeMainTextStateDefault3 />
      <SizeLargeTypeMainTextStateDefault4 />
    </div>
  );
}

function ProductDetails1() {
  return (
    <div className="bg-white flex-[1_0_0] min-h-px min-w-px relative rounded-[8px] self-stretch" data-name="Product Details">
      <div aria-hidden="true" className="absolute border border-[#eaeaea] border-solid inset-0 pointer-events-none rounded-[8px]" />
      <div className="content-stretch flex flex-col gap-[8px] items-start p-[16px] relative size-full">
        <Container3 />
        <Container4 />
      </div>
    </div>
  );
}

function Container() {
  return (
    <div className="content-stretch flex gap-[16px] h-[144px] items-start relative shrink-0 w-full" data-name="Container">
      <ProductDetails />
      <ProductDetails1 />
    </div>
  );
}

function Container6() {
  return (
    <div className="content-stretch flex gap-[16px] items-center relative shrink-0 w-full" data-name="Container">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Bold',sans-serif] font-bold justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[14px]">
        <p className="leading-[20px]">{`DMS connection information `}</p>
      </div>
    </div>
  );
}

function MainTextTrailingIcon5() {
  return (
    <div className="content-stretch flex gap-[4px] items-center min-w-[4px] relative shrink-0" data-name="Main text & trailing icon">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic relative shrink-0 text-[#656565] text-[14px] whitespace-nowrap">
        <p className="leading-[20px]">Quote #:</p>
      </div>
    </div>
  );
}

function SizeLargeTypeMainTextStateDefault5() {
  return (
    <div className="bg-white content-stretch flex gap-[8px] items-start py-[4px] relative shrink-0" data-name="Size=Large, Type=Main text, State=Default">
      <MainTextTrailingIcon5 />
    </div>
  );
}

function Container7() {
  return (
    <div className="content-stretch flex flex-col items-start relative shrink-0" data-name="Container">
      <SizeLargeTypeMainTextStateDefault5 />
    </div>
  );
}

function ProductDetails2() {
  return (
    <div className="bg-white flex-[1_0_0] h-[164px] min-h-px min-w-px relative rounded-[8px]" data-name="Product Details">
      <div aria-hidden="true" className="absolute border border-[#eaeaea] border-solid inset-0 pointer-events-none rounded-[8px]" />
      <div className="content-stretch flex flex-col gap-[8px] items-start p-[16px] relative size-full">
        <Container6 />
        <Container7 />
      </div>
    </div>
  );
}

function Container8() {
  return (
    <div className="content-stretch flex gap-[16px] items-center relative shrink-0" data-name="Container">
      <div className="flex flex-col font-['Inter:Bold',sans-serif] font-bold justify-center leading-[0] not-italic relative shrink-0 text-[#202020] text-[14px] whitespace-nowrap">
        <p className="leading-[20px]">Vehicle infromation</p>
      </div>
    </div>
  );
}

function MainTextTrailingIcon6() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center min-h-px min-w-[4px] relative" data-name="Main text & trailing icon">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#656565] text-[14px]">
        <p className="leading-[20px]">Owner:</p>
      </div>
    </div>
  );
}

function SizeLargeTypeMainTextStateDefault6() {
  return (
    <div className="bg-white content-stretch flex gap-[8px] items-start py-[4px] relative shrink-0 w-full" data-name="Size=Large, Type=Main text, State=Default">
      <MainTextTrailingIcon6 />
    </div>
  );
}

function MainTextTrailingIcon7() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center min-h-px min-w-[4px] relative" data-name="Main text & trailing icon">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[0px]">
        <p className="text-[14px]">
          <span className="leading-[20px] text-[#656565]">VIN</span>
          <span className="leading-[20px] text-[#656565]">:</span>
          <span className="leading-[20px] text-[#656565]">{` `}</span>
          <span className="leading-[20px]">{`1FTYR1ZM7KKA02839 `}</span>
        </p>
      </div>
    </div>
  );
}

function SizeLargeTypeMainTextStateDefault7() {
  return (
    <div className="bg-white content-stretch flex gap-[8px] items-start py-[4px] relative shrink-0 w-full" data-name="Size=Large, Type=Main text, State=Default">
      <MainTextTrailingIcon7 />
    </div>
  );
}

function MainTextTrailingIcon8() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center min-h-px min-w-[4px] relative" data-name="Main text & trailing icon">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[0px]">
        <p className="text-[14px]">
          <span className="leading-[20px] text-[#656565]">YMM</span>
          <span className="leading-[20px] text-[#656565]">:</span>
          <span className="leading-[20px] text-[#656565]">{` `}</span>
          <span className="leading-[20px]">2019 Ford Transit-250</span>
        </p>
      </div>
    </div>
  );
}

function SizeLargeTypeMainTextStateDefault8() {
  return (
    <div className="bg-white content-stretch flex gap-[8px] items-start py-[4px] relative shrink-0 w-full" data-name="Size=Large, Type=Main text, State=Default">
      <MainTextTrailingIcon8 />
    </div>
  );
}

function Container9() {
  return (
    <div className="content-stretch flex flex-col items-start relative shrink-0 w-full" data-name="Container">
      <SizeLargeTypeMainTextStateDefault6 />
      <SizeLargeTypeMainTextStateDefault7 />
      <SizeLargeTypeMainTextStateDefault8 />
    </div>
  );
}

function ProductDetails3() {
  return (
    <div className="bg-white flex-[1_0_0] min-h-px min-w-px relative rounded-[8px] self-stretch" data-name="Product Details">
      <div aria-hidden="true" className="absolute border border-[#eaeaea] border-solid inset-0 pointer-events-none rounded-[8px]" />
      <div className="content-stretch flex flex-col gap-[8px] items-start p-[16px] relative size-full">
        <Container8 />
        <Container9 />
      </div>
    </div>
  );
}

function Container5() {
  return (
    <div className="content-stretch flex gap-[16px] items-start relative shrink-0 w-full" data-name="Container">
      <ProductDetails2 />
      <ProductDetails3 />
    </div>
  );
}

function Container11() {
  return (
    <div className="content-stretch flex gap-[16px] items-center relative shrink-0" data-name="Container">
      <div className="flex flex-col font-['Inter:Bold',sans-serif] font-bold justify-center leading-[0] not-italic relative shrink-0 text-[#202020] text-[14px] whitespace-nowrap">
        <p className="leading-[20px]">Buyer information</p>
      </div>
    </div>
  );
}

function MainTextTrailingIcon9() {
  return (
    <div className="content-stretch flex gap-[4px] items-center min-w-[4px] relative shrink-0" data-name="Main text & trailing icon">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic relative shrink-0 text-[#202020] text-[0px] whitespace-nowrap">
        <p className="text-[14px]">
          <span className="leading-[20px] text-[#656565]">User name?</span>
          <span className="leading-[20px] text-[#656565]">:</span>
          <span className="leading-[20px] text-[#656565]">{` `}</span>
          <span className="leading-[20px]">Manju8888</span>
        </p>
      </div>
    </div>
  );
}

function SizeLargeTypeMainTextStateDefault9() {
  return (
    <div className="bg-white content-stretch flex gap-[8px] items-start py-[4px] relative shrink-0" data-name="Size=Large, Type=Main text, State=Default">
      <MainTextTrailingIcon9 />
    </div>
  );
}

function MainTextTrailingIcon10() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center min-h-px min-w-[4px] relative" data-name="Main text & trailing icon">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[0px]">
        <p className="text-[14px]">
          <span className="leading-[20px] text-[#656565]">Address</span>
          <span className="leading-[20px] text-[#656565]">:</span>
          <span className="leading-[20px] text-[#656565]">{` `}</span>
          <span className="leading-[20px]">OEC QA Repair Shop 88, 580 N, Columbus, OH 43215</span>
        </p>
      </div>
    </div>
  );
}

function SizeLargeTypeMainTextStateDefault10() {
  return (
    <div className="bg-white content-stretch flex gap-[8px] items-start py-[4px] relative shrink-0 w-full" data-name="Size=Large, Type=Main text, State=Default">
      <MainTextTrailingIcon10 />
    </div>
  );
}

function MainTextTrailingIcon11() {
  return (
    <div className="content-stretch flex gap-[4px] items-center min-w-[4px] relative shrink-0" data-name="Main text & trailing icon">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic relative shrink-0 text-[#202020] text-[0px] whitespace-nowrap">
        <p className="text-[14px]">
          <span className="leading-[20px] text-[#656565]">Phone</span>
          <span className="leading-[20px] text-[#656565]">:</span>
          <span className="leading-[20px] text-[#656565]">{` `}</span>
          <span className="leading-[20px]">(320) 252-5555</span>
        </p>
      </div>
    </div>
  );
}

function SizeLargeTypeMainTextStateDefault11() {
  return (
    <div className="bg-white content-stretch flex gap-[8px] items-start py-[4px] relative shrink-0" data-name="Size=Large, Type=Main text, State=Default">
      <MainTextTrailingIcon11 />
    </div>
  );
}

function MainTextTrailingIcon12() {
  return (
    <div className="content-stretch flex gap-[4px] items-center min-w-[4px] relative shrink-0" data-name="Main text & trailing icon">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic relative shrink-0 text-[#202020] text-[0px] whitespace-nowrap">
        <p className="text-[14px]">
          <span className="leading-[20px] text-[#656565]">Fax</span>
          <span className="leading-[20px] text-[#656565]">:</span>
          <span className="leading-[20px] text-[#656565]">{` `}</span>
          <span className="leading-[20px]">(320) 252-5555</span>
        </p>
      </div>
    </div>
  );
}

function SizeLargeTypeMainTextStateDefault12() {
  return (
    <div className="bg-white content-stretch flex gap-[8px] items-start py-[4px] relative shrink-0" data-name="Size=Large, Type=Main text, State=Default">
      <MainTextTrailingIcon12 />
    </div>
  );
}

function Container12() {
  return (
    <div className="content-stretch flex flex-col items-start relative shrink-0 w-full" data-name="Container">
      <SizeLargeTypeMainTextStateDefault9 />
      <SizeLargeTypeMainTextStateDefault10 />
      <SizeLargeTypeMainTextStateDefault11 />
      <SizeLargeTypeMainTextStateDefault12 />
    </div>
  );
}

function ProductDetails4() {
  return (
    <div className="bg-white flex-[1_0_0] min-h-px min-w-px relative rounded-[8px] self-stretch" data-name="Product Details">
      <div aria-hidden="true" className="absolute border border-[#eaeaea] border-solid inset-0 pointer-events-none rounded-[8px]" />
      <div className="content-stretch flex flex-col gap-[8px] items-start p-[16px] relative size-full">
        <Container11 />
        <Container12 />
      </div>
    </div>
  );
}

function Container13() {
  return (
    <div className="content-stretch flex gap-[16px] items-center relative shrink-0" data-name="Container">
      <div className="flex flex-col font-['Inter:Bold',sans-serif] font-bold justify-center leading-[0] not-italic relative shrink-0 text-[#202020] text-[14px] whitespace-nowrap">
        <p className="leading-[20px]">Transaction Information</p>
      </div>
    </div>
  );
}

function MainTextTrailingIcon13() {
  return (
    <div className="content-stretch flex gap-[4px] items-center min-w-[4px] relative shrink-0" data-name="Main text & trailing icon">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic relative shrink-0 text-[#161616] text-[0px] whitespace-nowrap">
        <p className="text-[#656565] text-[14px]">
          <span className="leading-[20px]">P.O#</span>
          <span className="leading-[20px]">:</span>
        </p>
      </div>
    </div>
  );
}

function SizeLargeTypeMainTextStateDefault13() {
  return (
    <div className="bg-white content-stretch flex gap-[8px] items-start py-[4px] relative shrink-0" data-name="Size=Large, Type=Main text, State=Default">
      <MainTextTrailingIcon13 />
    </div>
  );
}

function MainTextTrailingIcon14() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center min-h-px min-w-[4px] relative" data-name="Main text & trailing icon">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[0px]">
        <p className="text-[14px]">
          <span className="leading-[20px] text-[#656565]">Bill to</span>
          <span className="leading-[20px] text-[#656565]">:</span>
          <span className="leading-[20px] text-[#656565]">{` `}</span>
          <span className="leading-[20px]">OEC QA Repair Shop 88, 580 N, Columbus, OH 43215</span>
        </p>
      </div>
    </div>
  );
}

function SizeLargeTypeMainTextStateDefault14() {
  return (
    <div className="bg-white content-stretch flex gap-[8px] items-start py-[4px] relative shrink-0 w-full" data-name="Size=Large, Type=Main text, State=Default">
      <MainTextTrailingIcon14 />
    </div>
  );
}

function MainTextTrailingIcon15() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center min-h-px min-w-[4px] relative" data-name="Main text & trailing icon">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[0px]">
        <p className="text-[14px]">
          <span className="leading-[20px] text-[#656565]">Ship to</span>
          <span className="leading-[20px] text-[#656565]">:</span>
          <span className="leading-[20px] text-[#656565]">{` `}</span>
          <span className="leading-[20px]">OEC QA Repair Shop 88, 580 N, Columbus, OH 43215</span>
        </p>
      </div>
    </div>
  );
}

function SizeLargeTypeMainTextStateDefault15() {
  return (
    <div className="bg-white content-stretch flex gap-[8px] items-start py-[4px] relative shrink-0 w-full" data-name="Size=Large, Type=Main text, State=Default">
      <MainTextTrailingIcon15 />
    </div>
  );
}

function MainTextTrailingIcon16() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center min-h-px min-w-[4px] relative" data-name="Main text & trailing icon">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#161616] text-[0px]">
        <p className="text-[#656565] text-[14px]">
          <span className="leading-[20px]">Payment method</span>
          <span className="leading-[20px]">:</span>
          <span className="leading-[20px]">{` `}</span>
        </p>
      </div>
    </div>
  );
}

function SizeLargeTypeMainTextStateDefault16() {
  return (
    <div className="bg-white content-stretch flex gap-[8px] items-start py-[4px] relative shrink-0 w-full" data-name="Size=Large, Type=Main text, State=Default">
      <MainTextTrailingIcon16 />
    </div>
  );
}

function Container14() {
  return (
    <div className="content-stretch flex flex-col items-start relative shrink-0 w-full" data-name="Container">
      <SizeLargeTypeMainTextStateDefault13 />
      <SizeLargeTypeMainTextStateDefault14 />
      <SizeLargeTypeMainTextStateDefault15 />
      <SizeLargeTypeMainTextStateDefault16 />
    </div>
  );
}

function ProductDetails5() {
  return (
    <div className="bg-white flex-[1_0_0] h-[252px] min-h-px min-w-px relative rounded-[8px]" data-name="Product Details">
      <div aria-hidden="true" className="absolute border border-[#eaeaea] border-solid inset-0 pointer-events-none rounded-[8px]" />
      <div className="content-stretch flex flex-col gap-[8px] items-start p-[16px] relative size-full">
        <Container13 />
        <Container14 />
      </div>
    </div>
  );
}

function Container10() {
  return (
    <div className="content-stretch flex gap-[16px] items-start relative shrink-0 w-full" data-name="Container">
      <ProductDetails4 />
      <ProductDetails5 />
    </div>
  );
}

function Container16() {
  return (
    <div className="content-stretch flex gap-[16px] items-center relative shrink-0" data-name="Container">
      <div className="flex flex-col font-['Inter:Bold',sans-serif] font-bold justify-center leading-[0] not-italic relative shrink-0 text-[#202020] text-[14px] whitespace-nowrap">
        <p className="leading-[20px]">Buyer system Infromation</p>
      </div>
    </div>
  );
}

function ProductDetails6() {
  return (
    <div className="bg-white content-stretch flex flex-col h-[148px] items-start p-[16px] relative rounded-[8px] shrink-0 w-[326px]" data-name="Product Details">
      <div aria-hidden="true" className="absolute border border-[#eaeaea] border-solid inset-0 pointer-events-none rounded-[8px]" />
      <Container16 />
    </div>
  );
}

function Container15() {
  return (
    <div className="content-stretch flex items-start relative shrink-0 w-full" data-name="Container">
      <ProductDetails6 />
    </div>
  );
}

function Frame() {
  return (
    <div className="content-stretch flex flex-col gap-[16px] items-start relative shrink-0 w-full">
      <Container />
      <Container5 />
      <Container10 />
      <Container15 />
    </div>
  );
}

function Body() {
  return (
    <div className="bg-white relative shrink-0 w-full" data-name="Body">
      <div className="overflow-clip rounded-[inherit] size-full">
        <div className="content-stretch flex flex-col gap-[24px] items-start p-[24px] relative w-full">
          <div className="bg-white content-stretch flex h-[40px] items-start p-[4px] relative rounded-[4px] shrink-0 w-[635px]" data-name="Segmented control (In progress)">
            <div aria-hidden="true" className="absolute border border-[#c4c4c4] border-solid inset-0 pointer-events-none rounded-[4px]" />
            <div className="flex-[1_0_0] min-h-px min-w-px relative rounded-[4px]" data-name=".Segements">
              <div className="flex flex-row items-center justify-center size-full">
                <div className="content-stretch flex gap-[4px] items-center justify-center px-[8px] py-[4px] relative w-full">
                  <p className="font-['Inter:Bold',sans-serif] font-bold leading-[24px] not-italic relative shrink-0 text-[#003c71] text-[16px] whitespace-nowrap">VIN Details</p>
                </div>
              </div>
            </div>
            <div className="bg-[#003c71] flex-[1_0_0] min-h-px min-w-px relative rounded-[4px]" data-name=".Segements">
              <div className="flex flex-row items-center justify-center size-full">
                <div className="content-stretch flex gap-[4px] items-center justify-center px-[8px] py-[4px] relative w-full">
                  <p className="font-['Inter:Bold',sans-serif] font-bold leading-[24px] not-italic relative shrink-0 text-[16px] text-white whitespace-nowrap">Order Info</p>
                </div>
              </div>
            </div>
            <div className="flex-[1_0_0] min-h-px min-w-px relative rounded-[4px]" data-name=".Segements">
              <div className="flex flex-row items-center justify-center size-full">
                <div className="content-stretch flex gap-[4px] items-center justify-center px-[8px] py-[4px] relative w-full">
                  <p className="font-['Inter:Bold',sans-serif] font-bold leading-[24px] not-italic relative shrink-0 text-[#003c71] text-[16px] whitespace-nowrap">{`Photos & Notes`}</p>
                </div>
              </div>
            </div>
            <div className="flex-[1_0_0] min-h-px min-w-px relative rounded-[4px]" data-name=".Segements">
              <div className="flex flex-row items-center justify-center size-full">
                <div className="content-stretch flex gap-[4px] items-center justify-center px-[8px] py-[4px] relative w-full">
                  <p className="font-['Inter:Bold',sans-serif] font-bold leading-[24px] not-italic relative shrink-0 text-[#003c71] text-[16px] whitespace-nowrap">Event Log</p>
                </div>
              </div>
            </div>
          </div>
          <div className="flex flex-col font-['Inter:Bold',sans-serif] font-bold justify-end leading-[0] not-italic relative shrink-0 text-[#202020] text-[16px] whitespace-nowrap">
            <p className="leading-[24px]">Order Info</p>
          </div>
          <Frame />
        </div>
      </div>
    </div>
  );
}

export default function TableLineOrderInfo() {
  return (
    <div className="content-stretch flex flex-col items-start relative size-full" data-name="Table Line (Order Info)">
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
          <div className="bg-[#003c71] content-stretch flex items-center justify-center px-[16px] py-[8px] relative rounded-[8px] shrink-0" data-name="OButton">
            <p className="font-['Inter:Bold',sans-serif] font-bold leading-[24px] not-italic relative shrink-0 text-[16px] text-white whitespace-nowrap">Close</p>
          </div>
        </div>
      </div>
    </div>
  );
}