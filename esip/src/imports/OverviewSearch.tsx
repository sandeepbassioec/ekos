import svgPaths from "./svg-a8f9h7ftz9";
import imgScreenshot20250923At31410Pm2 from "figma:asset/42a7a1c41a09bb18dfa2ac80ca705b000f74660e.png";

function HeaderLink() {
  return (
    <div className="content-stretch flex gap-[8px] items-center relative shrink-0" data-name="Header link">
      <div className="flex flex-col font-['Arial:Bold',sans-serif] justify-end leading-[0] not-italic relative shrink-0 text-[16px] text-white whitespace-nowrap">
        <p className="leading-[24px]">CollisionLink</p>
      </div>
      <div className="bg-[#c9323c] content-stretch flex items-center justify-center px-[6px] py-[4px] relative rounded-[4px] shrink-0" data-name="Counter">
        <div className="flex flex-col font-['Arial:Bold',sans-serif] justify-center leading-[0] not-italic relative shrink-0 text-[12px] text-right text-white tracking-[0.25px] uppercase whitespace-nowrap">
          <p className="leading-[16px]">32</p>
        </div>
      </div>
    </div>
  );
}

function HeaderLink1() {
  return (
    <div className="content-stretch flex gap-[8px] items-center relative shrink-0" data-name="Header link">
      <div className="flex flex-col font-['Arial:Bold',sans-serif] justify-end leading-[0] not-italic relative shrink-0 text-[16px] text-white whitespace-nowrap">
        <p className="leading-[24px]">RepairLink</p>
      </div>
      <div className="bg-[#c9323c] content-stretch flex items-center justify-center px-[6px] py-[4px] relative rounded-[4px] shrink-0" data-name="Counter">
        <div className="flex flex-col font-['Arial:Bold',sans-serif] justify-center leading-[0] not-italic relative shrink-0 text-[12px] text-right text-white tracking-[0.25px] uppercase whitespace-nowrap">
          <p className="leading-[16px]">65</p>
        </div>
      </div>
    </div>
  );
}

function HeaderLinks() {
  return (
    <div className="content-stretch flex gap-[24px] items-center justify-end relative shrink-0 w-[515px]" data-name="Header links">
      <HeaderLink />
      <HeaderLink1 />
    </div>
  );
}

function HeaderContainer() {
  return (
    <div className="bg-[#16233d] relative shrink-0 w-full" data-name="Header container">
      <div className="flex flex-row items-center justify-end size-full">
        <div className="content-stretch flex items-center justify-end px-[24px] py-[12px] relative w-full">
          <HeaderLinks />
        </div>
      </div>
    </div>
  );
}

function SearchBar1() {
  return (
    <div className="bg-white content-stretch flex items-center px-[8px] py-[6px] relative shrink-0 w-[367px]" data-name="Search bar">
      <div aria-hidden="true" className="absolute border border-[#9da3b2] border-solid inset-0 pointer-events-none" />
      <div className="flex flex-col font-['Arial:Italic',sans-serif] justify-end leading-[0] not-italic relative shrink-0 text-[#666] text-[16px] whitespace-nowrap">
        <p className="leading-[24px]">Search to locate a part</p>
      </div>
    </div>
  );
}

function SearchBar2() {
  return (
    <div className="bg-white content-stretch flex items-center justify-between px-[8px] py-[6px] relative shrink-0 w-[160px]" data-name="Search bar">
      <div aria-hidden="true" className="absolute border border-[#9da3b2] border-solid inset-0 pointer-events-none" />
      <div className="flex flex-col font-['Arial:Regular',sans-serif] justify-end leading-[0] not-italic relative shrink-0 text-[#3c3c3c] text-[16px] whitespace-nowrap">
        <p className="leading-[24px]">Hyundai</p>
      </div>
      <div className="overflow-clip relative shrink-0 size-[24px]" data-name="Search dropdown">
        <div className="absolute bottom-[35.15%] left-1/4 right-[28.49%] top-[37.5%]" data-name="Vector">
          <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 11.163 6.56304">
            <path d={svgPaths.p3a221900} fill="var(--fill-0, #595959)" id="Vector" />
          </svg>
        </div>
      </div>
    </div>
  );
}

function Button() {
  return (
    <div className="bg-[#3376c3] content-stretch flex items-center justify-center px-[10px] py-[6px] relative shrink-0" data-name="Button">
      <div className="flex flex-col font-['Arial:Regular',sans-serif] justify-end leading-[0] not-italic relative shrink-0 text-[16px] text-white whitespace-nowrap">
        <p className="leading-[24px]">Locate</p>
      </div>
    </div>
  );
}

function SearchBar() {
  return (
    <div className="content-stretch flex gap-[8px] items-center relative shrink-0" data-name="Search bar">
      <SearchBar1 />
      <SearchBar2 />
      <Button />
    </div>
  );
}

function SearchContainer() {
  return (
    <div className="bg-white relative shrink-0 w-full" data-name="Search container">
      <div className="flex flex-row items-center size-full">
        <div className="content-stretch flex items-center justify-between px-[24px] py-[16px] relative w-full">
          <div className="h-[42px] relative shrink-0 w-[302px]" data-name="Screenshot 2025-09-23 at 3.14.10 PM 2">
            <div className="absolute inset-0 overflow-hidden pointer-events-none">
              <img alt="" className="absolute h-[1816.67%] left-[-5.3%] max-w-none top-[-126.19%] w-[490.73%]" src={imgScreenshot20250923At31410Pm2} />
            </div>
          </div>
          <SearchBar />
        </div>
      </div>
    </div>
  );
}

function NavigationItem() {
  return (
    <div className="bg-[#16233d] content-stretch flex items-center justify-center px-[8px] py-[4px] relative shrink-0" data-name="Navigation item">
      <div aria-hidden="true" className="absolute border-[#5eb3df] border-b-3 border-solid inset-[0_0_-3px_0] pointer-events-none" />
      <div className="flex flex-col font-['Arial:Bold',sans-serif] justify-end leading-[0] not-italic relative shrink-0 text-[16px] text-white whitespace-nowrap">
        <p className="leading-[24px]">OEC Home</p>
      </div>
    </div>
  );
}

function NavigationItem2() {
  return (
    <div className="bg-[#16233d] content-stretch flex items-center justify-center px-[8px] py-[4px] relative shrink-0" data-name="Navigation item">
      <div aria-hidden="true" className="absolute border-[#367b70] border-b-3 border-solid inset-[0_0_-3px_0] pointer-events-none" />
      <div className="flex flex-col font-['Arial:Bold',sans-serif] justify-end leading-[0] not-italic relative shrink-0 text-[16px] text-white whitespace-nowrap">
        <p className="leading-[24px]">LinkIQ</p>
      </div>
    </div>
  );
}

function NavigationItem1() {
  return (
    <div className="content-stretch flex items-start relative shrink-0" data-name="Navigation item">
      <div className="bg-[#6a81ac] h-[35px] shrink-0 w-px" data-name="Navigation icon" />
      <NavigationItem2 />
    </div>
  );
}

function NavigationItem4() {
  return (
    <div className="bg-[#16233d] content-stretch flex items-center justify-center px-[8px] py-[4px] relative shrink-0" data-name="Navigation item">
      <div aria-hidden="true" className="absolute border-[#bf612b] border-b-3 border-solid inset-[0_0_-3px_0] pointer-events-none" />
      <div className="flex flex-col font-['Arial:Bold',sans-serif] justify-end leading-[0] not-italic relative shrink-0 text-[16px] text-white whitespace-nowrap">
        <p className="leading-[24px]">D2DLink</p>
      </div>
    </div>
  );
}

function NavigationItem3() {
  return (
    <div className="content-stretch flex items-start relative shrink-0" data-name="Navigation item">
      <div className="bg-[#6a81ac] h-[35px] shrink-0 w-px" data-name="Navigation icon" />
      <NavigationItem4 />
    </div>
  );
}

function NavigationItem6() {
  return (
    <div className="bg-[#16233d] content-stretch flex items-center justify-center px-[8px] py-[4px] relative shrink-0" data-name="Navigation item">
      <div aria-hidden="true" className="absolute border-[#bf612b] border-b-3 border-solid inset-[0_0_-3px_0] pointer-events-none" />
      <div className="flex flex-col font-['Arial:Bold',sans-serif] justify-end leading-[0] not-italic relative shrink-0 text-[16px] text-white whitespace-nowrap">
        <p className="leading-[24px]">D2D Express</p>
      </div>
    </div>
  );
}

function NavigationItem5() {
  return (
    <div className="content-stretch flex items-start relative shrink-0" data-name="Navigation item">
      <div className="bg-[#6a81ac] h-[35px] shrink-0 w-px" data-name="Navigation icon" />
      <NavigationItem6 />
    </div>
  );
}

function NavigationItem8() {
  return (
    <div className="bg-[#16233d] content-stretch flex items-center justify-center px-[8px] py-[4px] relative shrink-0" data-name="Navigation item">
      <div aria-hidden="true" className="absolute border-[#b02b40] border-b-3 border-solid inset-[0_0_-3px_0] pointer-events-none" />
      <div className="flex flex-col font-['Arial:Bold',sans-serif] justify-end leading-[0] not-italic relative shrink-0 text-[16px] text-white whitespace-nowrap">
        <p className="leading-[24px]">CollisionLink</p>
      </div>
    </div>
  );
}

function NavigationItem7() {
  return (
    <div className="content-stretch flex items-start relative shrink-0" data-name="Navigation item">
      <div className="bg-[#6a81ac] h-[35px] shrink-0 w-px" data-name="Navigation icon" />
      <NavigationItem8 />
    </div>
  );
}

function NavigationItem10() {
  return (
    <div className="bg-[#16233d] content-stretch flex items-center justify-center px-[8px] py-[4px] relative shrink-0" data-name="Navigation item">
      <div aria-hidden="true" className="absolute border-[#db4733] border-b-3 border-solid inset-[0_0_-3px_0] pointer-events-none" />
      <div className="flex flex-col font-['Arial:Bold',sans-serif] justify-end leading-[0] not-italic relative shrink-0 text-[16px] text-white whitespace-nowrap">
        <p className="leading-[24px]">MyPriceLink</p>
      </div>
    </div>
  );
}

function NavigationItem9() {
  return (
    <div className="content-stretch flex items-start relative shrink-0" data-name="Navigation item">
      <div className="bg-[#6a81ac] h-[35px] shrink-0 w-px" data-name="Navigation icon" />
      <NavigationItem10 />
    </div>
  );
}

function NavigationItem12() {
  return (
    <div className="bg-[#5b8fd6] content-stretch flex items-center justify-center px-[8px] py-[4px] relative shrink-0" data-name="Navigation item">
      <div aria-hidden="true" className="absolute border-[#3e6293] border-b-3 border-solid inset-[0_0_-3px_0] pointer-events-none" />
      <div className="flex flex-col font-['Arial:Bold',sans-serif] justify-end leading-[0] not-italic relative shrink-0 text-[16px] text-white whitespace-nowrap">
        <p className="leading-[24px]">RepairLink</p>
      </div>
    </div>
  );
}

function NavigationItem11() {
  return (
    <div className="content-stretch flex items-start relative shrink-0" data-name="Navigation item">
      <div className="bg-[#6a81ac] h-[35px] shrink-0 w-px" data-name="Navigation icon" />
      <NavigationItem12 />
    </div>
  );
}

function NavigationItem14() {
  return (
    <div className="bg-[#16233d] content-stretch flex items-center justify-center px-[8px] py-[4px] relative shrink-0" data-name="Navigation item">
      <div aria-hidden="true" className="absolute border-[#367b70] border-b-3 border-solid inset-[0_0_-3px_0] pointer-events-none" />
      <div className="flex flex-col font-['Arial:Bold',sans-serif] justify-end leading-[0] not-italic relative shrink-0 text-[20px] text-white whitespace-nowrap">
        <p className="leading-[24px]">MarketplaceDirect</p>
      </div>
    </div>
  );
}

function NavigationItem13() {
  return (
    <div className="content-stretch flex items-start relative shrink-0" data-name="Navigation item">
      <div className="bg-[#6a81ac] h-[35px] shrink-0 w-px" data-name="Navigation icon" />
      <NavigationItem14 />
    </div>
  );
}

function NavigationItem16() {
  return (
    <div className="bg-[#16233d] content-stretch flex items-center justify-center px-[8px] py-[4px] relative shrink-0" data-name="Navigation item">
      <div aria-hidden="true" className="absolute border-[#562757] border-b-3 border-solid inset-[0_0_-3px_0] pointer-events-none" />
      <div className="flex flex-col font-['Arial:Bold',sans-serif] justify-end leading-[0] not-italic relative shrink-0 text-[16px] text-white whitespace-nowrap">
        <p className="leading-[24px]">ConsumerLink</p>
      </div>
    </div>
  );
}

function NavigationItem15() {
  return (
    <div className="content-stretch flex items-start relative shrink-0" data-name="Navigation item">
      <div className="bg-[#6a81ac] h-[35px] shrink-0 w-px" data-name="Navigation icon" />
      <NavigationItem16 />
    </div>
  );
}

function NavigationItem18() {
  return (
    <div className="bg-[#16233d] content-stretch flex items-center justify-center px-[8px] py-[4px] relative shrink-0" data-name="Navigation item">
      <div aria-hidden="true" className="absolute border-[#b02b40] border-b-3 border-solid inset-[0_0_-3px_0] pointer-events-none" />
      <div className="flex flex-col font-['Arial:Bold',sans-serif] justify-end leading-[0] not-italic relative shrink-0 text-[16px] text-white whitespace-nowrap">
        <p className="leading-[24px]">DeliverNow</p>
      </div>
    </div>
  );
}

function NavigationItem17() {
  return (
    <div className="content-stretch flex items-start relative shrink-0" data-name="Navigation item">
      <div className="bg-[#6a81ac] h-[35px] shrink-0 w-px" data-name="Navigation icon" />
      <NavigationItem18 />
    </div>
  );
}

function NavigationItem20() {
  return (
    <div className="bg-[#16233d] content-stretch flex items-center justify-center px-[8px] py-[4px] relative shrink-0" data-name="Navigation item">
      <div aria-hidden="true" className="absolute border-[#367b70] border-b-3 border-solid inset-[0_0_-3px_0] pointer-events-none" />
      <div className="flex flex-col font-['Arial:Bold',sans-serif] justify-end leading-[0] not-italic relative shrink-0 text-[16px] text-white whitespace-nowrap">
        <p className="leading-[24px]">Payments</p>
      </div>
    </div>
  );
}

function NavigationItem19() {
  return (
    <div className="content-stretch flex items-start relative shrink-0" data-name="Navigation item">
      <div className="bg-[#6a81ac] h-[35px] shrink-0 w-px" data-name="Navigation icon" />
      <NavigationItem20 />
    </div>
  );
}

function NavigationItem22() {
  return (
    <div className="bg-[#16233d] content-stretch flex items-center justify-center px-[8px] py-[4px] relative shrink-0" data-name="Navigation item">
      <div aria-hidden="true" className="absolute border-[#bf612b] border-b-3 border-solid inset-[0_0_-3px_0] pointer-events-none" />
      <div className="flex flex-col font-['Arial:Bold',sans-serif] justify-end leading-[0] not-italic relative shrink-0 text-[16px] text-white whitespace-nowrap">
        <p className="leading-[24px]">PastBrokerDirect</p>
      </div>
    </div>
  );
}

function NavigationItem21() {
  return (
    <div className="content-stretch flex items-start relative shrink-0" data-name="Navigation item">
      <div className="bg-[#6a81ac] h-[35px] shrink-0 w-px" data-name="Navigation icon" />
      <NavigationItem22 />
    </div>
  );
}

function NavigationBar() {
  return (
    <div className="content-stretch flex items-start relative shrink-0" data-name="Navigation bar">
      <NavigationItem />
      <NavigationItem1 />
      <NavigationItem3 />
      <NavigationItem5 />
      <NavigationItem7 />
      <NavigationItem9 />
      <NavigationItem11 />
      <NavigationItem13 />
      <NavigationItem15 />
      <NavigationItem17 />
      <NavigationItem19 />
      <NavigationItem21 />
    </div>
  );
}

function Header() {
  return (
    <div className="absolute content-stretch flex flex-col items-start left-0 top-0 w-[1440px]" data-name="Header">
      <HeaderContainer />
      <SearchContainer />
      <NavigationBar />
    </div>
  );
}

function SidebarItem() {
  return (
    <div className="bg-[#424d62] relative shrink-0 w-full" data-name="Sidebar item">
      <div className="flex flex-row items-center size-full">
        <div className="content-stretch flex items-center px-[16px] py-[8px] relative w-full">
          <div className="flex flex-col font-['Arial:Bold',sans-serif] justify-end leading-[0] not-italic relative shrink-0 text-[14px] text-white whitespace-nowrap">
            <p className="leading-[20px]">RepairLink</p>
          </div>
        </div>
      </div>
    </div>
  );
}

function SidebarItem1() {
  return (
    <div className="relative shrink-0 w-full" data-name="Sidebar item">
      <div aria-hidden="true" className="absolute border-[#6a81ac] border-b-[0.5px] border-solid inset-0 pointer-events-none" />
      <div className="flex flex-row items-center size-full">
        <div className="content-stretch flex items-center p-[8px] relative w-full">
          <div className="flex flex-col font-['Arial:Regular',sans-serif] justify-end leading-[0] not-italic relative shrink-0 text-[#869bc4] text-[14px] whitespace-nowrap">
            <p className="leading-[20px]">RepairLink Tools</p>
          </div>
          <div className="overflow-clip relative shrink-0 size-[24px]" data-name="Sidebar dropdown">
            <div className="absolute bottom-[35.15%] left-1/4 right-[28.49%] top-[37.5%]" data-name="Vector">
              <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 11.163 6.56304">
                <path d={svgPaths.p3a221900} fill="var(--fill-0, #869BC4)" id="Vector" />
              </svg>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function SidebarItem2() {
  return (
    <div className="relative shrink-0 w-full" data-name="Sidebar item">
      <div aria-hidden="true" className="absolute border-[#6a81ac] border-b-[0.5px] border-solid inset-0 pointer-events-none" />
      <div className="flex flex-row items-center size-full">
        <div className="content-stretch flex items-center px-[16px] py-[8px] relative w-full">
          <div className="flex flex-col font-['Arial:Regular',sans-serif] justify-end leading-[0] not-italic relative shrink-0 text-[14px] text-white whitespace-nowrap">
            <p className="leading-[20px]">Program Settlement</p>
          </div>
        </div>
      </div>
    </div>
  );
}

function SidebarItem3() {
  return (
    <div className="relative shrink-0 w-full" data-name="Sidebar item">
      <div aria-hidden="true" className="absolute border-[#6a81ac] border-b-[0.5px] border-solid inset-0 pointer-events-none" />
      <div className="flex flex-row items-center size-full">
        <div className="content-stretch flex items-center px-[16px] py-[8px] relative w-full">
          <div className="flex flex-col font-['Arial:Regular',sans-serif] justify-end leading-[0] not-italic relative shrink-0 text-[14px] text-white whitespace-nowrap">
            <p className="leading-[20px]">View Archive</p>
          </div>
        </div>
      </div>
    </div>
  );
}

function SidebarItem4() {
  return (
    <div className="relative shrink-0 w-full" data-name="Sidebar item">
      <div aria-hidden="true" className="absolute border-[#6a81ac] border-b-[0.5px] border-solid inset-0 pointer-events-none" />
      <div className="flex flex-row items-center size-full">
        <div className="content-stretch flex items-center px-[16px] py-[8px] relative w-full">
          <div className="flex flex-col font-['Arial:Regular',sans-serif] justify-end leading-[0] not-italic relative shrink-0 text-[14px] text-white whitespace-nowrap">
            <p className="leading-[20px]">Promotion Setup</p>
          </div>
        </div>
      </div>
    </div>
  );
}

function SidebarItem5() {
  return (
    <div className="relative shrink-0 w-full" data-name="Sidebar item">
      <div aria-hidden="true" className="absolute border-[#6a81ac] border-b-[0.5px] border-solid inset-0 pointer-events-none" />
      <div className="flex flex-row items-center size-full">
        <div className="content-stretch flex items-center px-[16px] py-[8px] relative w-full">
          <div className="flex flex-col font-['Arial:Regular',sans-serif] justify-end leading-[0] not-italic relative shrink-0 text-[14px] text-white whitespace-nowrap">
            <p className="leading-[20px]">Return Parts</p>
          </div>
        </div>
      </div>
    </div>
  );
}

function SidebarItem6() {
  return (
    <div className="relative shrink-0 w-full" data-name="Sidebar item">
      <div aria-hidden="true" className="absolute border-[#6a81ac] border-b-[0.5px] border-solid inset-0 pointer-events-none" />
      <div className="flex flex-row items-center size-full">
        <div className="content-stretch flex items-center p-[8px] relative w-full">
          <div className="flex flex-col font-['Arial:Regular',sans-serif] justify-end leading-[0] not-italic relative shrink-0 text-[#869bc4] text-[14px] whitespace-nowrap">
            <p className="leading-[20px]">Setup</p>
          </div>
          <div className="overflow-clip relative shrink-0 size-[24px]" data-name="Sidebar dropdown">
            <div className="absolute bottom-[35.15%] left-1/4 right-[28.49%] top-[37.5%]" data-name="Vector">
              <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 11.163 6.56304">
                <path d={svgPaths.p3a221900} fill="var(--fill-0, #869BC4)" id="Vector" />
              </svg>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function SidebarItem7() {
  return (
    <div className="relative shrink-0 w-full" data-name="Sidebar item">
      <div aria-hidden="true" className="absolute border-[#6a81ac] border-b-[0.5px] border-solid inset-0 pointer-events-none" />
      <div className="flex flex-row items-center size-full">
        <div className="content-stretch flex items-center px-[16px] py-[8px] relative w-full">
          <div className="flex flex-col font-['Arial:Regular',sans-serif] justify-end leading-[0] not-italic relative shrink-0 text-[14px] text-white whitespace-nowrap">
            <p className="leading-[20px]">Applications Setup</p>
          </div>
        </div>
      </div>
    </div>
  );
}

function Sidebar() {
  return (
    <div className="absolute bg-[#16233d] content-stretch flex flex-col h-[1014px] items-start left-0 top-[157px] w-[180px]" data-name="Sidebar">
      <SidebarItem />
      <SidebarItem1 />
      <SidebarItem2 />
      <SidebarItem3 />
      <SidebarItem4 />
      <SidebarItem5 />
      <SidebarItem6 />
      <SidebarItem7 />
    </div>
  );
}

function Text() {
  return (
    <div className="content-stretch flex gap-[4px] items-center justify-center px-[12px] relative shrink-0" data-name="Text">
      <div className="overflow-clip relative shrink-0 size-[24px]" data-name="Checkmark">
        <div className="absolute inset-[10.01%_10%_10%_10.01%]" data-name="Vector">
          <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 19.1966 19.1981">
            <path d={svgPaths.p2997f700} fill="var(--fill-0, #003C71)" id="Vector" />
          </svg>
        </div>
      </div>
      <p className="font-['Inter:Bold',sans-serif] font-bold leading-[24px] not-italic relative shrink-0 text-[#003c71] text-[16px] text-center whitespace-nowrap">Overview</p>
    </div>
  );
}

function Text1() {
  return (
    <div className="content-stretch flex gap-[4px] items-center justify-center px-[12px] relative shrink-0" data-name="Text">
      <div className="overflow-clip relative shrink-0 size-[24px]" data-name="Checkmark">
        <div className="absolute inset-[20%_10%]" data-name="Vector">
          <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 19.2 14.4">
            <path d={svgPaths.p3ec28d00} fill="var(--fill-0, #4D4E53)" id="Vector" />
          </svg>
        </div>
      </div>
      <p className="font-['Inter:Bold',sans-serif] font-bold leading-[24px] not-italic relative shrink-0 text-[#4d4e53] text-[16px] text-center whitespace-nowrap">Transaction</p>
    </div>
  );
}

function CarouselTabs() {
  return (
    <div className="bg-white content-stretch flex gap-[8px] items-center justify-center px-[12px] py-[8px] relative rounded-[100px] shrink-0" data-name="Carousel tabs">
      <div aria-hidden="true" className="absolute border border-[#bfbfbf] border-solid inset-0 pointer-events-none rounded-[100px]" />
      <div className="overflow-clip relative shrink-0 size-[24px]" data-name="Checkmark">
        <div className="absolute inset-[20%_10%_10%_9.99%]" data-name="Vector">
          <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 19.2011 16.7992">
            <path d={svgPaths.p3cfcf280} fill="var(--fill-0, #4D4E53)" id="Vector" />
          </svg>
        </div>
      </div>
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic relative shrink-0 text-[#202020] text-[14px] whitespace-nowrap">
        <p className="leading-[20px]">Filters</p>
      </div>
    </div>
  );
}

function TextInput() {
  return (
    <div className="content-stretch flex items-center relative shrink-0" data-name="Text Input">
      <p className="font-['Inter:Regular',sans-serif] font-normal leading-[24px] not-italic relative shrink-0 text-[#202020] text-[16px] whitespace-nowrap">P</p>
      <div className="bg-[#1b9dd9] h-[24px] shrink-0 w-px" data-name="Cursor" />
    </div>
  );
}

function InputContainer() {
  return (
    <div className="content-stretch flex gap-[8px] items-center relative shrink-0" data-name="Input Container">
      <div className="overflow-clip relative shrink-0 size-[24px]" data-name="Search Icon">
        <div className="absolute inset-[10%_10%_9.99%_10%]" data-name="Vector">
          <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 19.1991 19.2028">
            <path d={svgPaths.p30863d80} fill="var(--fill-0, #4D4E53)" id="Vector" />
          </svg>
        </div>
      </div>
      <TextInput />
    </div>
  );
}

function CarouselTab() {
  return (
    <div className="content-stretch flex gap-[16px] items-center relative shrink-0" data-name="Carousel tab">
      <CarouselTabs />
      <div className="content-stretch flex items-center justify-between px-[16px] py-[8px] relative rounded-[8px] shrink-0 w-[300px]" data-name="Search field">
        <div aria-hidden="true" className="absolute border-2 border-[#1b9dd9] border-solid inset-0 pointer-events-none rounded-[8px]" />
        <InputContainer />
        <div className="overflow-clip relative shrink-0 size-[24px]" data-name="Clear Icon">
          <div className="absolute inset-[10%]" data-name="Vector">
            <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 19.2 19.2">
              <path d={svgPaths.p34604980} fill="var(--fill-0, #4D4E53)" id="Vector" />
            </svg>
          </div>
        </div>
      </div>
    </div>
  );
}

function Container1() {
  return (
    <div className="bg-white content-stretch flex flex-col gap-[16px] items-start p-[16px] relative rounded-[8px] shrink-0 w-[291px]" data-name="Container">
      <div aria-hidden="true" className="absolute border border-[#eaeaea] border-solid inset-0 pointer-events-none rounded-[8px]" />
      <div className="flex flex-col font-['Arial:Bold',sans-serif] justify-center leading-[0] not-italic relative shrink-0 text-[#bd5f15] text-[20px] tracking-[0.15px] whitespace-nowrap">
        <p className="leading-[28px]">14 Pending Orders</p>
      </div>
      <div className="bg-[#003c71] content-stretch flex items-center justify-center px-[16px] py-[8px] relative rounded-[8px] shrink-0" data-name="OButton">
        <p className="font-['Inter:Bold',sans-serif] font-bold leading-[24px] not-italic relative shrink-0 text-[16px] text-white whitespace-nowrap">Manage</p>
      </div>
    </div>
  );
}

function Container2() {
  return (
    <div className="bg-white content-stretch flex flex-col gap-[12px] items-start p-[16px] relative rounded-[8px] shrink-0 w-[291px]" data-name="Container">
      <div aria-hidden="true" className="absolute border border-[#eaeaea] border-solid inset-0 pointer-events-none rounded-[8px]" />
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic relative shrink-0 text-[#656565] text-[16px] whitespace-nowrap">
        <p className="leading-[24px]">Part Qty. Sold</p>
      </div>
      <div className="flex flex-col font-['Inter:Bold',sans-serif] font-bold justify-center leading-[0] min-w-full not-italic relative shrink-0 text-[#202020] text-[20px] w-[min-content]">
        <p className="leading-[28px]">154</p>
      </div>
    </div>
  );
}

function Container3() {
  return (
    <div className="bg-white content-stretch flex flex-col gap-[12px] items-start p-[16px] relative rounded-[8px] shrink-0 w-[291px]" data-name="Container">
      <div aria-hidden="true" className="absolute border border-[#eaeaea] border-solid inset-0 pointer-events-none rounded-[8px]" />
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic relative shrink-0 text-[#656565] text-[16px] whitespace-nowrap">
        <p className="leading-[24px]">PMA Parts in Stock</p>
      </div>
      <div className="flex flex-col font-['Inter:Bold',sans-serif] font-bold justify-center leading-[0] min-w-full not-italic relative shrink-0 text-[#202020] text-[20px] w-[min-content]">
        <p className="leading-[28px]">195</p>
      </div>
    </div>
  );
}

function Frame() {
  return (
    <div className="content-stretch flex gap-[16px] items-start relative shrink-0">
      <Container2 />
      <Container3 />
    </div>
  );
}

function Frame1() {
  return (
    <div className="content-stretch flex flex-col gap-[16px] items-start relative shrink-0">
      <Container1 />
      <Frame />
    </div>
  );
}

function Container() {
  return (
    <div className="absolute content-stretch flex flex-col gap-[24px] items-start left-[204px] overflow-clip top-[181px] w-[1212px]" data-name="Container">
      <div className="content-stretch flex items-center relative rounded-br-[8px] rounded-tr-[8px] shrink-0" data-name="Tab">
        <div className="bg-white content-stretch flex flex-col gap-[8px] items-start pt-[8px] relative shrink-0" data-name="Tab Elements">
          <Text />
          <div className="bg-[#003c71] h-[2px] rounded-[999px] shrink-0 w-full" data-name="Underline" />
        </div>
        <div className="bg-white content-stretch flex flex-col gap-[8px] items-start pt-[8px] relative shrink-0" data-name="Tab Elements">
          <Text1 />
          <div className="bg-[#bfbfbf] h-[2px] shrink-0 w-full" data-name="Underline" />
        </div>
      </div>
      <CarouselTab />
      <Frame1 />
    </div>
  );
}

export default function OverviewSearch() {
  return (
    <div className="bg-white relative size-full" data-name="Overview (search)">
      <Header />
      <Sidebar />
      <Container />
    </div>
  );
}