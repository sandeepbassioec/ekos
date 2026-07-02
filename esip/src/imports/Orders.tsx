import svgPaths from "./svg-sgjzi9z8l8";
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
    <div className="absolute content-stretch flex flex-col items-start left-0 top-0 w-[2681px]" data-name="Header">
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
    <div className="absolute bg-[#16233d] content-stretch flex flex-col h-[1225px] items-start left-0 top-[157px] w-[180px]" data-name="Sidebar">
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

function Frame23() {
  return (
    <div className="content-stretch flex flex-col items-start relative shrink-0">
      <p className="font-['Inter:Bold',sans-serif] font-bold leading-[24px] min-w-full not-italic relative shrink-0 text-[#4d4e53] text-[16px] w-[min-content]">Order Details</p>
    </div>
  );
}

function LeftSection() {
  return (
    <div className="content-stretch flex gap-[8px] items-center relative shrink-0" data-name="Left Section">
      <div className="overflow-clip relative shrink-0 size-[24px]" data-name="Back Icon">
        <div className="-translate-y-1/2 absolute aspect-[8/14] left-[33.34%] right-[33.33%] top-[calc(50%+0.6px)]" data-name="Vector">
          <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 8 14">
            <path d={svgPaths.p9367df0} fill="var(--fill-0, #4D4E53)" id="Vector" />
          </svg>
        </div>
      </div>
      <Frame23 />
    </div>
  );
}

function Frame24() {
  return (
    <div className="content-stretch flex gap-[8px] items-center justify-end relative shrink-0">
      <div className="content-stretch flex gap-[8px] items-center relative shrink-0" data-name="Header Right Block">
        <div className="bg-white content-stretch flex gap-[4px] h-[32px] items-center justify-center px-[16px] py-[8px] relative rounded-[8px] shrink-0" data-name="OButton">
          <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / box-archive">
            <div className="absolute inset-[15%_10%]" data-name="Vector">
              <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.8 11.2">
                <path d={svgPaths.p24983500} fill="var(--fill-0, #003C71)" id="Vector" />
              </svg>
            </div>
          </div>
          <p className="font-['Inter:Bold',sans-serif] font-bold leading-[20px] not-italic relative shrink-0 text-[#003c71] text-[14px] whitespace-nowrap">Archive</p>
        </div>
        <div className="bg-white content-stretch flex gap-[4px] h-[32px] items-center justify-center px-[16px] py-[8px] relative rounded-[8px] shrink-0" data-name="OButton">
          <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / print">
            <div className="absolute inset-[10%]" data-name="Vector">
              <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.8 12.8">
                <path d={svgPaths.peef8400} fill="var(--fill-0, #003C71)" id="Vector" />
              </svg>
            </div>
          </div>
          <p className="font-['Inter:Bold',sans-serif] font-bold leading-[20px] not-italic relative shrink-0 text-[#003c71] text-[14px] whitespace-nowrap">Print</p>
        </div>
      </div>
    </div>
  );
}

function Container3() {
  return (
    <div className="flex-[1_0_0] min-h-px min-w-px relative w-[245.555px]" data-name="Container">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid content-stretch flex gap-[12px] items-center relative size-full">
        <p className="font-['Inter:Bold',sans-serif] font-bold leading-[28px] not-italic relative shrink-0 text-[#202020] text-[20px] whitespace-nowrap">Transaction Status:</p>
        <div className="bg-[#f3f8f4] relative rounded-[999px] shrink-0" data-name="Tag">
          <div aria-hidden="true" className="absolute border border-[#3a6f3b] border-solid inset-0 pointer-events-none rounded-[999px]" />
          <div className="bg-clip-padding border-0 border-[transparent] border-solid content-stretch flex gap-[4px] items-center px-[8px] py-[4px] relative">
            <p className="font-['Inter:Bold',sans-serif] font-bold leading-[16px] not-italic relative shrink-0 text-[#202020] text-[12px] whitespace-nowrap">New</p>
          </div>
        </div>
      </div>
    </div>
  );
}

function Container2() {
  return (
    <div className="h-[53.141px] relative shrink-0 w-[245.555px]" data-name="Container">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid content-stretch flex flex-col gap-[8px] items-start relative size-full">
        <Container3 />
        <p className="font-['Inter:Regular',sans-serif] font-normal leading-[0] not-italic relative shrink-0 text-[#c50505] text-[0px] text-[14px] whitespace-nowrap">
          <span className="leading-[20px] text-[#656565]">Transaction ID:</span>
          <span className="leading-[20px]">{` `}</span>
          <span className="font-['Inter:Bold',sans-serif] font-bold leading-[20px] text-[#202020]">2736273</span>
        </p>
      </div>
    </div>
  );
}

function Container1() {
  return (
    <div className="content-stretch flex items-center justify-between relative shrink-0 w-full" data-name="Container">
      <Container2 />
      <div className="bg-white relative rounded-[8px] shrink-0" data-name="OButton">
        <div className="bg-clip-padding border-0 border-[transparent] border-solid content-stretch flex gap-[4px] items-center justify-center px-[16px] py-[8px] relative">
          <p className="font-['Inter:Bold',sans-serif] font-bold leading-[24px] not-italic relative shrink-0 text-[#003c71] text-[16px] whitespace-nowrap">View more details</p>
          <div className="overflow-clip relative shrink-0 size-[24px]" data-name="Icon / chevron-right">
            <div className="-translate-y-1/2 absolute aspect-[8/14] left-[35.07%] right-[31.6%] top-[calc(50%+0.6px)]" data-name="Vector">
              <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 8 14">
                <path d={svgPaths.p287f9b00} fill="var(--fill-0, #003C71)" id="Vector" />
              </svg>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function Left() {
  return (
    <div className="content-stretch flex gap-[4px] items-center relative shrink-0" data-name="Left">
      <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / circle-info">
        <div className="absolute inset-[10%]" data-name="Vector">
          <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.8 12.8">
            <path d={svgPaths.p18411100} fill="var(--fill-0, #4D4E53)" id="Vector" />
          </svg>
        </div>
      </div>
      <p className="font-['Inter:Regular',sans-serif] font-normal leading-[20px] not-italic relative shrink-0 text-[#202020] text-[14px] whitespace-nowrap">Buyer asked to use their saved payment method for the charge</p>
    </div>
  );
}

function Container4() {
  return (
    <div className="bg-white flex-[1_0_0] h-[70px] min-h-px min-w-px relative rounded-[8px]" data-name="Container">
      <div aria-hidden="true" className="absolute border border-[#eaeaea] border-solid inset-0 pointer-events-none rounded-[8px]" />
      <div className="flex flex-col justify-center size-full">
        <div className="content-stretch flex flex-col items-start justify-center p-[13px] relative size-full">
          <p className="font-['Inter:Regular',sans-serif] font-normal leading-[0] not-italic relative shrink-0 text-[#c50505] text-[0px] text-[14px] whitespace-nowrap">
            <span className="leading-[20px] text-[#656565]">Payment status:</span>
            <span className="leading-[20px]">{` `}</span>
            <span className="leading-[20px] text-[#202020]">Unpaid</span>
          </p>
        </div>
      </div>
    </div>
  );
}

function Container5() {
  return (
    <div className="bg-white flex-[1_0_0] min-h-px min-w-px relative rounded-[8px]" data-name="Container">
      <div aria-hidden="true" className="absolute border border-[#eaeaea] border-solid inset-0 pointer-events-none rounded-[8px]" />
      <div className="flex flex-col justify-center size-full">
        <div className="content-stretch flex flex-col font-['Inter:Regular',sans-serif] font-normal gap-[4px] items-start justify-center leading-[0] not-italic p-[13px] relative text-[#949494] text-[0px] text-[14px] w-full whitespace-nowrap">
          <p className="relative shrink-0">
            <span className="leading-[20px] text-[#656565]">VIN:</span>
            <span className="leading-[20px]">{` `}</span>
            <span className="leading-[20px] text-[#202020]">1SKDSDSNDSDNSDKNSD</span>
          </p>
          <p className="relative shrink-0">
            <span className="leading-[20px] text-[#656565]">YMM:</span>
            <span className="leading-[20px]">{` `}</span>
            <span className="leading-[20px] text-[#202020]">2019 Ford Transit - 250</span>
          </p>
        </div>
      </div>
    </div>
  );
}

function Container6() {
  return (
    <div className="bg-white flex-[1_0_0] min-h-px min-w-px relative rounded-[8px]" data-name="Container">
      <div aria-hidden="true" className="absolute border border-[#eaeaea] border-solid inset-0 pointer-events-none rounded-[8px]" />
      <div className="flex flex-col justify-center size-full">
        <div className="content-stretch flex flex-col font-['Inter:Regular',sans-serif] font-normal gap-[4px] items-start justify-center leading-[0] not-italic p-[13px] relative text-[#949494] text-[0px] text-[14px] w-full whitespace-nowrap">
          <p className="relative shrink-0">
            <span className="leading-[20px] text-[#656565]">Buyer:</span>
            <span className="leading-[20px]">{` `}</span>
            <span className="leading-[20px] text-[#202020]">Test Account</span>
          </p>
          <p className="relative shrink-0">
            <span className="leading-[20px] text-[#656565]">DMS Cust #:</span>
            <span className="leading-[20px]">{` `}</span>
          </p>
        </div>
      </div>
    </div>
  );
}

function Frame3() {
  return (
    <div className="relative shrink-0">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid content-stretch flex gap-[8px] items-center relative">
        <div className="overflow-clip relative shrink-0 size-[20px]" data-name="Icon / link-simple">
          <div className="absolute inset-[20%_5%]" data-name="Vector">
            <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 18 12">
              <path d={svgPaths.p1fe3b780} fill="var(--fill-0, #656565)" id="Vector" />
            </svg>
          </div>
        </div>
        <p className="font-['Inter:Regular',sans-serif] font-normal leading-[0] not-italic relative shrink-0 text-[#949494] text-[0px] text-[14px] whitespace-nowrap">
          <span className="leading-[20px] text-[#656565]">Photos:</span>
          <span className="leading-[20px]">{` `}</span>
          <span className="leading-[20px] text-[#202020]">{`Total (4) · `}</span>
          <span className="leading-[20px] text-[#3a6f3b]">New (2)</span>
        </p>
      </div>
    </div>
  );
}

function Frame4() {
  return (
    <div className="relative shrink-0">
      <div className="bg-clip-padding border-0 border-[transparent] border-solid content-stretch flex gap-[8px] items-center relative">
        <div className="overflow-clip relative shrink-0 size-[20px]" data-name="Icon / link-simple">
          <div className="absolute inset-[20%_5%]" data-name="Vector">
            <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 18 12">
              <path d={svgPaths.p1fe3b780} fill="var(--fill-0, #656565)" id="Vector" />
            </svg>
          </div>
        </div>
        <p className="font-['Inter:Regular',sans-serif] font-normal leading-[0] not-italic relative shrink-0 text-[#949494] text-[0px] text-[14px] whitespace-nowrap">
          <span className="leading-[20px] text-[#656565]">Photos:</span>
          <span className="leading-[20px]">{` `}</span>
          <span className="leading-[20px] text-[#202020]">{`Total (4) · `}</span>
          <span className="leading-[20px] text-[#3a6f3b]">New (2)</span>
        </p>
      </div>
    </div>
  );
}

function Container7() {
  return (
    <div className="bg-white flex-[1_0_0] min-h-px min-w-px relative rounded-[8px]" data-name="Container">
      <div aria-hidden="true" className="absolute border border-[#eaeaea] border-solid inset-0 pointer-events-none rounded-[8px]" />
      <div className="flex flex-col justify-center size-full">
        <div className="content-stretch flex flex-col gap-[4px] items-start justify-center p-[13px] relative w-full">
          <Frame3 />
          <Frame4 />
        </div>
      </div>
    </div>
  );
}

function Frame5() {
  return (
    <div className="content-stretch flex gap-[16px] items-center relative shrink-0 w-full">
      <Container4 />
      <Container5 />
      <Container6 />
      <Container7 />
    </div>
  );
}

function Frame2() {
  return (
    <div className="content-stretch flex items-center justify-between relative shrink-0 w-full">
      <p className="font-['Inter:Regular',sans-serif] font-normal leading-[20px] not-italic relative shrink-0 text-[#656565] text-[14px] whitespace-nowrap">QOH was last updated: 06:52 AM ET on 10/09/2025</p>
      <div className="bg-white content-stretch flex gap-[4px] items-center justify-center px-[16px] py-[8px] relative rounded-[8px] shrink-0" data-name="OButton">
        <div className="overflow-clip relative shrink-0 size-[24px]" data-name="Icon / arrow-rotate-left">
          <div className="absolute inset-[10%]" data-name="Vector">
            <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 19.2 19.2">
              <path d={svgPaths.pc155a00} fill="var(--fill-0, #003C71)" id="Vector" />
            </svg>
          </div>
        </div>
        <p className="font-['Inter:Bold',sans-serif] font-bold leading-[24px] not-italic relative shrink-0 text-[#003c71] text-[16px] whitespace-nowrap">Refresh</p>
      </div>
    </div>
  );
}

function ProductDetails() {
  return (
    <div className="bg-white content-stretch flex flex-col gap-[16px] items-start p-[16px] relative rounded-[12px] shrink-0 w-[1104px]" data-name="ProductDetails">
      <div aria-hidden="true" className="absolute border border-[#eaeaea] border-solid inset-0 pointer-events-none rounded-[12px]" />
      <Container1 />
      <div className="bg-white content-stretch flex gap-[8px] items-center p-[8px] relative rounded-[6px] shrink-0" data-name="Banner">
        <div aria-hidden="true" className="absolute border border-[#dcdcdc] border-solid inset-0 pointer-events-none rounded-[6px]" />
        <Left />
      </div>
      <Frame5 />
      <div className="bg-[#eaeaea] h-px shrink-0 w-full" data-name="Divider Line" />
      <Frame2 />
    </div>
  );
}

function Frame1() {
  return (
    <div className="content-stretch flex items-center relative shrink-0">
      <div className="bg-white content-stretch flex gap-[4px] items-center justify-center px-[16px] py-[8px] relative rounded-[8px] shrink-0" data-name="OButton">
        <div className="overflow-clip relative shrink-0 size-[24px]" data-name="Icon / location-dot">
          <div className="absolute inset-[10%_20%_9.53%_19.98%]" data-name="Vector">
            <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 14.4038 19.3125">
              <path d={svgPaths.p292a6180} fill="var(--fill-0, #003C71)" id="Vector" />
            </svg>
          </div>
        </div>
        <p className="font-['Inter:Bold',sans-serif] font-bold leading-[24px] not-italic relative shrink-0 text-[#003c71] text-[16px] whitespace-nowrap">Locate part</p>
      </div>
      <div className="bg-white content-stretch flex gap-[4px] items-center justify-center px-[16px] py-[8px] relative rounded-[8px] shrink-0" data-name="OButton">
        <div className="overflow-clip relative shrink-0 size-[24px]" data-name="Icon / plus">
          <div className="absolute inset-[15%]" data-name="Vector">
            <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16.8 16.8">
              <path d={svgPaths.pcf2e00} fill="var(--fill-0, #003C71)" id="Vector" />
            </svg>
          </div>
        </div>
        <p className="font-['Inter:Bold',sans-serif] font-bold leading-[24px] not-italic relative shrink-0 text-[#003c71] text-[16px] whitespace-nowrap">Add part</p>
      </div>
    </div>
  );
}

function Frame() {
  return (
    <div className="content-stretch flex items-center justify-end relative shrink-0 w-full">
      <Frame1 />
    </div>
  );
}

function Header1() {
  return (
    <div className="content-stretch flex gap-[4px] items-center relative shrink-0" data-name="Header">
      <div className="flex flex-col font-['Inter:Bold',sans-serif] font-bold justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#0b0b0b] text-[14px] text-ellipsis whitespace-nowrap">
        <p className="leading-[20px] overflow-hidden">EPC</p>
      </div>
    </div>
  );
}

function Left1() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[8px] items-center min-h-px min-w-px relative" data-name="Left">
      <Header1 />
    </div>
  );
}

function Column() {
  return (
    <div className="bg-white content-stretch flex flex-col items-start relative shrink-0 w-[80px]" data-name="Column">
      <div className="bg-[#eaeaea] content-stretch flex gap-[8px] h-[48px] items-center p-[16px] relative shrink-0 w-[280px]" data-name=".Header">
        <Left1 />
        <div className="-translate-y-1/2 absolute bg-[rgba(24,29,31,0.15)] content-stretch flex h-[14px] items-center left-0 overflow-clip top-1/2 w-[2px]" data-name="Pipe - L">
          <div className="bg-[rgba(24,29,31,0.15)] h-full shrink-0 w-[2px]" data-name="shape" />
        </div>
      </div>
      <div className="relative shrink-0 w-full" data-name=".Accessory cell">
        <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
        <div className="flex flex-row items-center size-full">
          <div className="content-stretch flex items-center px-[16px] py-[12px] relative w-full">
            <div className="overflow-clip relative shrink-0 size-[24px]" data-name="Icon / arrow-up-right-from-square">
              <div className="absolute inset-[10%]" data-name="Vector">
                <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 19.2 19.2">
                  <path d={svgPaths.p1f634f00} fill="var(--fill-0, #003C71)" id="Vector" />
                </svg>
              </div>
            </div>
          </div>
        </div>
      </div>
      <div className="relative shrink-0 w-full" data-name=".Accessory cell">
        <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
        <div className="flex flex-row items-center size-full">
          <div className="content-stretch flex items-center px-[16px] py-[12px] relative w-full">
            <div className="overflow-clip relative shrink-0 size-[24px]" data-name="Icon / arrow-up-right-from-square">
              <div className="absolute inset-[10%]" data-name="Vector">
                <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 19.2 19.2">
                  <path d={svgPaths.p1f634f00} fill="var(--fill-0, #003C71)" id="Vector" />
                </svg>
              </div>
            </div>
          </div>
        </div>
      </div>
      <div className="relative shrink-0 w-full" data-name=".Accessory cell">
        <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
        <div className="flex flex-row items-center size-full">
          <div className="content-stretch flex items-center px-[16px] py-[12px] relative w-full">
            <div className="overflow-clip relative shrink-0 size-[24px]" data-name="Icon / arrow-up-right-from-square">
              <div className="absolute inset-[10%]" data-name="Vector">
                <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 19.2 19.2">
                  <path d={svgPaths.p1f634f00} fill="var(--fill-0, #003C71)" id="Vector" />
                </svg>
              </div>
            </div>
          </div>
        </div>
      </div>
      <div className="relative shrink-0 w-full" data-name=".Accessory cell">
        <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
        <div className="flex flex-row items-center size-full">
          <div className="content-stretch flex items-center px-[16px] py-[12px] relative w-full">
            <div className="overflow-clip relative shrink-0 size-[24px]" data-name="Icon / arrow-up-right-from-square">
              <div className="absolute inset-[10%]" data-name="Vector">
                <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 19.2 19.2">
                  <path d={svgPaths.p1f634f00} fill="var(--fill-0, #003C71)" id="Vector" />
                </svg>
              </div>
            </div>
          </div>
        </div>
      </div>
      <div className="relative shrink-0 w-full" data-name=".Accessory cell">
        <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
        <div className="flex flex-row items-center size-full">
          <div className="content-stretch flex items-center px-[16px] py-[12px] relative w-full">
            <div className="overflow-clip relative shrink-0 size-[24px]" data-name="Icon / arrow-up-right-from-square">
              <div className="absolute inset-[10%]" data-name="Vector">
                <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 19.2 19.2">
                  <path d={svgPaths.p1f634f00} fill="var(--fill-0, #003C71)" id="Vector" />
                </svg>
              </div>
            </div>
          </div>
        </div>
      </div>
      <div className="relative shrink-0 w-full" data-name=".Accessory cell">
        <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
        <div className="flex flex-row items-center size-full">
          <div className="content-stretch flex items-center px-[16px] py-[12px] relative w-full">
            <div className="overflow-clip relative shrink-0 size-[24px]" data-name="Icon / arrow-up-right-from-square">
              <div className="absolute inset-[10%]" data-name="Vector">
                <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 19.2 19.2">
                  <path d={svgPaths.p1f634f00} fill="var(--fill-0, #003C71)" id="Vector" />
                </svg>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function Header2() {
  return (
    <div className="content-stretch flex gap-[4px] items-center relative shrink-0" data-name="Header">
      <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / circle-info">
        <div className="absolute inset-[10%]" data-name="Vector">
          <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.8 12.8">
            <path d={svgPaths.p18411100} fill="var(--fill-0, #4D4E53)" id="Vector" />
          </svg>
        </div>
      </div>
      <div className="flex flex-col font-['Inter:Bold',sans-serif] font-bold justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#0b0b0b] text-[14px] text-ellipsis whitespace-nowrap">
        <p className="leading-[20px] overflow-hidden">Part Number</p>
      </div>
    </div>
  );
}

function Left2() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[8px] items-center min-h-px min-w-px relative" data-name="Left">
      <Header2 />
      <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / arrow-up">
        <div className="absolute inset-[11.25%_19.99%_8.75%_19.99%]" data-name="Vector">
          <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 9.60375 12.7994">
            <path d={svgPaths.p2b66af00} fill="var(--fill-0, #4D4E53)" id="Vector" />
          </svg>
        </div>
      </div>
    </div>
  );
}

function Label() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">RepairLink</p>
      </div>
    </div>
  );
}

function Label1() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">RepairLink</p>
      </div>
    </div>
  );
}

function Label2() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">RepairLink</p>
      </div>
    </div>
  );
}

function Label3() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">RepairLink</p>
      </div>
    </div>
  );
}

function Label4() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">RepairLink</p>
      </div>
    </div>
  );
}

function Label5() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">RepairLink</p>
      </div>
    </div>
  );
}

function Header3() {
  return (
    <div className="content-stretch flex gap-[4px] items-center relative shrink-0" data-name="Header">
      <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / circle-info">
        <div className="absolute inset-[10%]" data-name="Vector">
          <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.8 12.8">
            <path d={svgPaths.p18411100} fill="var(--fill-0, #4D4E53)" id="Vector" />
          </svg>
        </div>
      </div>
      <div className="flex flex-col font-['Inter:Bold',sans-serif] font-bold justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#0b0b0b] text-[14px] text-ellipsis whitespace-nowrap">
        <p className="leading-[20px] overflow-hidden">Invoice #</p>
      </div>
    </div>
  );
}

function Left3() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[8px] items-center min-h-px min-w-px relative" data-name="Left">
      <Header3 />
    </div>
  );
}

function Label6() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">Memo</p>
      </div>
    </div>
  );
}

function Label7() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">Memo</p>
      </div>
    </div>
  );
}

function Label8() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">Memo</p>
      </div>
    </div>
  );
}

function Label9() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">Memo</p>
      </div>
    </div>
  );
}

function Label10() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">Memo</p>
      </div>
    </div>
  );
}

function Label11() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">Memo</p>
      </div>
    </div>
  );
}

function Header4() {
  return (
    <div className="content-stretch flex gap-[4px] items-center relative shrink-0" data-name="Header">
      <div className="flex flex-col font-['Inter:Bold',sans-serif] font-bold justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#0b0b0b] text-[14px] text-ellipsis whitespace-nowrap">
        <p className="leading-[20px] overflow-hidden">Status</p>
      </div>
    </div>
  );
}

function Left4() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[8px] items-center min-h-px min-w-px relative" data-name="Left">
      <Header4 />
    </div>
  );
}

function Label12() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">Memo</p>
      </div>
    </div>
  );
}

function Label13() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">Memo</p>
      </div>
    </div>
  );
}

function Label14() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">Memo</p>
      </div>
    </div>
  );
}

function Label15() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">Memo</p>
      </div>
    </div>
  );
}

function Label16() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">Memo</p>
      </div>
    </div>
  );
}

function Label17() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">Memo</p>
      </div>
    </div>
  );
}

function Column1() {
  return (
    <div className="bg-white content-stretch flex flex-col items-start relative self-stretch shrink-0 w-[216px]" data-name="Column">
      <div className="bg-[#eaeaea] h-[48px] relative shrink-0 w-full" data-name=".Header">
        <div className="flex flex-row items-center size-full">
          <div className="content-stretch flex gap-[8px] items-center p-[16px] relative size-full">
            <Left4 />
            <div className="-translate-y-1/2 absolute bg-[rgba(24,29,31,0.15)] content-stretch flex h-[14px] items-center left-0 overflow-clip top-1/2 w-[2px]" data-name="Pipe - L">
              <div className="bg-[rgba(24,29,31,0.15)] h-full shrink-0 w-[2px]" data-name="shape" />
            </div>
          </div>
        </div>
      </div>
      <div className="relative shrink-0 w-full" data-name=".Interactive cell">
        <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
        <div className="flex flex-row items-center size-full">
          <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative w-full">
            <Label12 />
            <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / chevron-down">
              <div className="-translate-y-1/2 absolute aspect-[14/8] left-[23.32%] right-[18.35%] top-[calc(50%+0.4px)]" data-name="Vector">
                <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 9.33333 5.33333">
                  <path d={svgPaths.p2fc97600} fill="var(--fill-0, #4D4E53)" id="Vector" />
                </svg>
              </div>
            </div>
          </div>
        </div>
      </div>
      <div className="relative shrink-0 w-full" data-name=".Interactive cell">
        <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
        <div className="flex flex-row items-center size-full">
          <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative w-full">
            <Label13 />
            <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / chevron-down">
              <div className="-translate-y-1/2 absolute aspect-[14/8] left-[23.32%] right-[18.35%] top-[calc(50%+0.4px)]" data-name="Vector">
                <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 9.33333 5.33333">
                  <path d={svgPaths.p2fc97600} fill="var(--fill-0, #4D4E53)" id="Vector" />
                </svg>
              </div>
            </div>
          </div>
        </div>
      </div>
      <div className="relative shrink-0 w-full" data-name=".Interactive cell">
        <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
        <div className="flex flex-row items-center size-full">
          <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative w-full">
            <Label14 />
            <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / chevron-down">
              <div className="-translate-y-1/2 absolute aspect-[14/8] left-[23.32%] right-[18.35%] top-[calc(50%+0.4px)]" data-name="Vector">
                <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 9.33333 5.33333">
                  <path d={svgPaths.p2fc97600} fill="var(--fill-0, #4D4E53)" id="Vector" />
                </svg>
              </div>
            </div>
          </div>
        </div>
      </div>
      <div className="relative shrink-0 w-full" data-name=".Interactive cell">
        <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
        <div className="flex flex-row items-center size-full">
          <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative w-full">
            <Label15 />
            <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / chevron-down">
              <div className="-translate-y-1/2 absolute aspect-[14/8] left-[23.32%] right-[18.35%] top-[calc(50%+0.4px)]" data-name="Vector">
                <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 9.33333 5.33333">
                  <path d={svgPaths.p2fc97600} fill="var(--fill-0, #4D4E53)" id="Vector" />
                </svg>
              </div>
            </div>
          </div>
        </div>
      </div>
      <div className="relative shrink-0 w-full" data-name=".Interactive cell">
        <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
        <div className="flex flex-row items-center size-full">
          <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative w-full">
            <Label16 />
            <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / chevron-down">
              <div className="-translate-y-1/2 absolute aspect-[14/8] left-[23.32%] right-[18.35%] top-[calc(50%+0.4px)]" data-name="Vector">
                <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 9.33333 5.33333">
                  <path d={svgPaths.p2fc97600} fill="var(--fill-0, #4D4E53)" id="Vector" />
                </svg>
              </div>
            </div>
          </div>
        </div>
      </div>
      <div className="relative shrink-0 w-full" data-name=".Interactive cell">
        <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
        <div className="flex flex-row items-center size-full">
          <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative w-full">
            <Label17 />
            <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / chevron-down">
              <div className="-translate-y-1/2 absolute aspect-[14/8] left-[23.32%] right-[18.35%] top-[calc(50%+0.4px)]" data-name="Vector">
                <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 9.33333 5.33333">
                  <path d={svgPaths.p2fc97600} fill="var(--fill-0, #4D4E53)" id="Vector" />
                </svg>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function Header5() {
  return (
    <div className="content-stretch flex gap-[4px] items-center relative shrink-0" data-name="Header">
      <div className="flex flex-col font-['Inter:Bold',sans-serif] font-bold justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#0b0b0b] text-[14px] text-ellipsis whitespace-nowrap">
        <p className="leading-[20px] overflow-hidden">Program</p>
      </div>
    </div>
  );
}

function Left5() {
  return (
    <div className="content-stretch flex gap-[8px] items-center relative shrink-0" data-name="Left">
      <Header5 />
      <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / arrow-up">
        <div className="absolute inset-[11.25%_19.99%_8.75%_19.99%]" data-name="Vector">
          <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 9.60375 12.7994">
            <path d={svgPaths.p2b66af00} fill="var(--fill-0, #4D4E53)" id="Vector" />
          </svg>
        </div>
      </div>
    </div>
  );
}

function Clip() {
  return (
    <div className="h-[20px] overflow-clip relative shrink-0 w-[16px]" data-name="Clip">
      <p className="-translate-x-1/2 absolute font-['Inter:Bold',sans-serif] font-bold h-[19px] leading-[20px] left-[8px] not-italic text-[14px] text-center text-white top-0 w-[16px]">MW</p>
    </div>
  );
}

function Clip1() {
  return (
    <div className="h-[20px] overflow-clip relative shrink-0 w-[16px]" data-name="Clip">
      <p className="-translate-x-1/2 absolute font-['Inter:Bold',sans-serif] font-bold h-[19px] leading-[20px] left-[8px] not-italic text-[14px] text-center text-white top-0 w-[16px]">MW</p>
    </div>
  );
}

function Clip2() {
  return (
    <div className="h-[20px] overflow-clip relative shrink-0 w-[16px]" data-name="Clip">
      <p className="-translate-x-1/2 absolute font-['Inter:Bold',sans-serif] font-bold h-[19px] leading-[20px] left-[8px] not-italic text-[14px] text-center text-white top-0 w-[16px]">MW</p>
    </div>
  );
}

function Clip3() {
  return (
    <div className="h-[20px] overflow-clip relative shrink-0 w-[16px]" data-name="Clip">
      <p className="-translate-x-1/2 absolute font-['Inter:Bold',sans-serif] font-bold h-[19px] leading-[20px] left-[8px] not-italic text-[14px] text-center text-white top-0 w-[16px]">MW</p>
    </div>
  );
}

function Clip4() {
  return (
    <div className="h-[20px] overflow-clip relative shrink-0 w-[16px]" data-name="Clip">
      <p className="-translate-x-1/2 absolute font-['Inter:Bold',sans-serif] font-bold h-[19px] leading-[20px] left-[8px] not-italic text-[14px] text-center text-white top-0 w-[16px]">MW</p>
    </div>
  );
}

function Clip5() {
  return (
    <div className="h-[20px] overflow-clip relative shrink-0 w-[16px]" data-name="Clip">
      <p className="-translate-x-1/2 absolute font-['Inter:Bold',sans-serif] font-bold h-[19px] leading-[20px] left-[8px] not-italic text-[14px] text-center text-white top-0 w-[16px]">MW</p>
    </div>
  );
}

function Column2() {
  return (
    <div className="bg-white content-stretch flex flex-col items-start relative self-stretch shrink-0 w-[120px]" data-name="Column">
      <div className="bg-[#eaeaea] h-[48px] relative shrink-0 w-full" data-name=".Header">
        <div className="flex flex-row items-center size-full">
          <div className="content-stretch flex gap-[8px] items-center p-[16px] relative size-full">
            <Left5 />
            <div className="-translate-y-1/2 absolute bg-[rgba(24,29,31,0.15)] content-stretch flex h-[14px] items-center left-0 overflow-clip top-1/2 w-[2px]" data-name="Pipe - L">
              <div className="bg-[rgba(24,29,31,0.15)] h-full shrink-0 w-[2px]" data-name="shape" />
            </div>
          </div>
        </div>
      </div>
      <div className="relative shrink-0 w-full" data-name=".Accessory cell">
        <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
        <div className="flex flex-row items-center size-full">
          <div className="content-stretch flex items-center px-[16px] py-[8px] relative w-full">
            <div className="bg-[#003c71] content-stretch flex flex-col items-center justify-center relative rounded-[999px] shrink-0 size-[32px]" data-name="Avatar">
              <Clip />
            </div>
          </div>
        </div>
      </div>
      <div className="relative shrink-0 w-full" data-name=".Accessory cell">
        <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
        <div className="flex flex-row items-center size-full">
          <div className="content-stretch flex items-center px-[16px] py-[8px] relative w-full">
            <div className="bg-[#003c71] content-stretch flex flex-col items-center justify-center relative rounded-[999px] shrink-0 size-[32px]" data-name="Avatar">
              <Clip1 />
            </div>
          </div>
        </div>
      </div>
      <div className="relative shrink-0 w-full" data-name=".Accessory cell">
        <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
        <div className="flex flex-row items-center size-full">
          <div className="content-stretch flex items-center px-[16px] py-[8px] relative w-full">
            <div className="bg-[#003c71] content-stretch flex flex-col items-center justify-center relative rounded-[999px] shrink-0 size-[32px]" data-name="Avatar">
              <Clip2 />
            </div>
          </div>
        </div>
      </div>
      <div className="relative shrink-0 w-full" data-name=".Accessory cell">
        <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
        <div className="flex flex-row items-center size-full">
          <div className="content-stretch flex items-center px-[16px] py-[8px] relative w-full">
            <div className="bg-[#003c71] content-stretch flex flex-col items-center justify-center relative rounded-[999px] shrink-0 size-[32px]" data-name="Avatar">
              <Clip3 />
            </div>
          </div>
        </div>
      </div>
      <div className="relative shrink-0 w-full" data-name=".Accessory cell">
        <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
        <div className="flex flex-row items-center size-full">
          <div className="content-stretch flex items-center px-[16px] py-[8px] relative w-full">
            <div className="bg-[#003c71] content-stretch flex flex-col items-center justify-center relative rounded-[999px] shrink-0 size-[32px]" data-name="Avatar">
              <Clip4 />
            </div>
          </div>
        </div>
      </div>
      <div className="relative shrink-0 w-full" data-name=".Accessory cell">
        <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
        <div className="flex flex-row items-center size-full">
          <div className="content-stretch flex items-center px-[16px] py-[8px] relative w-full">
            <div className="bg-[#003c71] content-stretch flex flex-col items-center justify-center relative rounded-[999px] shrink-0 size-[32px]" data-name="Avatar">
              <Clip5 />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function Header6() {
  return (
    <div className="content-stretch flex gap-[4px] items-center relative shrink-0" data-name="Header">
      <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / circle-info">
        <div className="absolute inset-[10%]" data-name="Vector">
          <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.8 12.8">
            <path d={svgPaths.p18411100} fill="var(--fill-0, #4D4E53)" id="Vector" />
          </svg>
        </div>
      </div>
      <div className="flex flex-col font-['Inter:Bold',sans-serif] font-bold justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#0b0b0b] text-[14px] text-ellipsis whitespace-nowrap">
        <p className="leading-[20px] overflow-hidden">Description</p>
      </div>
    </div>
  );
}

function Left6() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[8px] items-center min-h-px min-w-px relative" data-name="Left">
      <Header6 />
    </div>
  );
}

function Label18() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">Airbag control module</p>
      </div>
    </div>
  );
}

function Label19() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">Airbag control module</p>
      </div>
    </div>
  );
}

function Label20() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">Airbag control module</p>
      </div>
    </div>
  );
}

function Label21() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">Airbag control module</p>
      </div>
    </div>
  );
}

function Label22() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">Airbag control module</p>
      </div>
    </div>
  );
}

function Label23() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">Airbag control module</p>
      </div>
    </div>
  );
}

function Header7() {
  return (
    <div className="content-stretch flex gap-[4px] items-center relative shrink-0" data-name="Header">
      <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / circle-info">
        <div className="absolute inset-[10%]" data-name="Vector">
          <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.8 12.8">
            <path d={svgPaths.p18411100} fill="var(--fill-0, #4D4E53)" id="Vector" />
          </svg>
        </div>
      </div>
      <div className="flex flex-col font-['Inter:Bold',sans-serif] font-bold justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#0b0b0b] text-[14px] text-ellipsis whitespace-nowrap">
        <p className="leading-[20px] overflow-hidden">Requested</p>
      </div>
    </div>
  );
}

function Left7() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[8px] items-center min-h-px min-w-px relative" data-name="Left">
      <Header7 />
    </div>
  );
}

function Label24() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">1</p>
      </div>
    </div>
  );
}

function Label25() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">1</p>
      </div>
    </div>
  );
}

function Label26() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">1</p>
      </div>
    </div>
  );
}

function Label27() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">1</p>
      </div>
    </div>
  );
}

function Label28() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">1</p>
      </div>
    </div>
  );
}

function Label29() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">1</p>
      </div>
    </div>
  );
}

function Header8() {
  return (
    <div className="content-stretch flex gap-[4px] items-center relative shrink-0" data-name="Header">
      <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / circle-info">
        <div className="absolute inset-[10%]" data-name="Vector">
          <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.8 12.8">
            <path d={svgPaths.p18411100} fill="var(--fill-0, #4D4E53)" id="Vector" />
          </svg>
        </div>
      </div>
      <div className="flex flex-col font-['Inter:Bold',sans-serif] font-bold justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#0b0b0b] text-[14px] text-ellipsis whitespace-nowrap">
        <p className="leading-[20px] overflow-hidden">On Hand</p>
      </div>
    </div>
  );
}

function Left8() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[8px] items-center min-h-px min-w-px relative" data-name="Left">
      <Header8 />
    </div>
  );
}

function Label30() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">0</p>
      </div>
    </div>
  );
}

function Label31() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">0</p>
      </div>
    </div>
  );
}

function Label32() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">0</p>
      </div>
    </div>
  );
}

function Label33() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">0</p>
      </div>
    </div>
  );
}

function Label34() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">0</p>
      </div>
    </div>
  );
}

function Label35() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">0</p>
      </div>
    </div>
  );
}

function Header9() {
  return (
    <div className="content-stretch flex gap-[4px] items-center relative shrink-0" data-name="Header">
      <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / circle-info">
        <div className="absolute inset-[10%]" data-name="Vector">
          <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.8 12.8">
            <path d={svgPaths.p18411100} fill="var(--fill-0, #4D4E53)" id="Vector" />
          </svg>
        </div>
      </div>
      <div className="flex flex-col font-['Inter:Bold',sans-serif] font-bold justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#0b0b0b] text-[14px] text-ellipsis whitespace-nowrap">
        <p className="leading-[20px] overflow-hidden">Revised Part #</p>
      </div>
    </div>
  );
}

function Left9() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[8px] items-center min-h-px min-w-px relative" data-name="Left">
      <Header9 />
    </div>
  );
}

function Label36() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">236526</p>
      </div>
    </div>
  );
}

function Label37() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">236526</p>
      </div>
    </div>
  );
}

function Label38() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">236526</p>
      </div>
    </div>
  );
}

function Label39() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">236526</p>
      </div>
    </div>
  );
}

function Label40() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">236526</p>
      </div>
    </div>
  );
}

function Label41() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">236526</p>
      </div>
    </div>
  );
}

function Header10() {
  return (
    <div className="content-stretch flex gap-[4px] items-center relative shrink-0" data-name="Header">
      <div className="flex flex-col font-['Inter:Bold',sans-serif] font-bold justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#0b0b0b] text-[14px] text-ellipsis whitespace-nowrap">
        <p className="leading-[20px] overflow-hidden">Delivery Option</p>
      </div>
    </div>
  );
}

function Left10() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[8px] items-center min-h-px min-w-px relative" data-name="Left">
      <Header10 />
    </div>
  );
}

function Label42() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">Standart</p>
      </div>
    </div>
  );
}

function Label43() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">Standart</p>
      </div>
    </div>
  );
}

function Label44() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">Standart</p>
      </div>
    </div>
  );
}

function Label45() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">Standart</p>
      </div>
    </div>
  );
}

function Label46() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">Standart</p>
      </div>
    </div>
  );
}

function Label47() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">Standart</p>
      </div>
    </div>
  );
}

function Column3() {
  return (
    <div className="bg-white content-stretch flex flex-col items-start relative self-stretch shrink-0 w-[216px]" data-name="Column">
      <div className="bg-[#eaeaea] h-[48px] relative shrink-0 w-full" data-name=".Header">
        <div className="flex flex-row items-center size-full">
          <div className="content-stretch flex gap-[8px] items-center p-[16px] relative size-full">
            <Left10 />
            <div className="-translate-y-1/2 absolute bg-[rgba(24,29,31,0.15)] content-stretch flex h-[14px] items-center left-0 overflow-clip top-1/2 w-[2px]" data-name="Pipe - L">
              <div className="bg-[rgba(24,29,31,0.15)] h-full shrink-0 w-[2px]" data-name="shape" />
            </div>
          </div>
        </div>
      </div>
      <div className="relative shrink-0 w-full" data-name=".Interactive cell">
        <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
        <div className="flex flex-row items-center size-full">
          <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative w-full">
            <Label42 />
            <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / chevron-down">
              <div className="-translate-y-1/2 absolute aspect-[14/8] left-[23.32%] right-[18.35%] top-[calc(50%+0.4px)]" data-name="Vector">
                <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 9.33333 5.33333">
                  <path d={svgPaths.p2fc97600} fill="var(--fill-0, #4D4E53)" id="Vector" />
                </svg>
              </div>
            </div>
          </div>
        </div>
      </div>
      <div className="relative shrink-0 w-full" data-name=".Interactive cell">
        <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
        <div className="flex flex-row items-center size-full">
          <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative w-full">
            <Label43 />
            <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / chevron-down">
              <div className="-translate-y-1/2 absolute aspect-[14/8] left-[23.32%] right-[18.35%] top-[calc(50%+0.4px)]" data-name="Vector">
                <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 9.33333 5.33333">
                  <path d={svgPaths.p2fc97600} fill="var(--fill-0, #4D4E53)" id="Vector" />
                </svg>
              </div>
            </div>
          </div>
        </div>
      </div>
      <div className="relative shrink-0 w-full" data-name=".Interactive cell">
        <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
        <div className="flex flex-row items-center size-full">
          <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative w-full">
            <Label44 />
            <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / chevron-down">
              <div className="-translate-y-1/2 absolute aspect-[14/8] left-[23.32%] right-[18.35%] top-[calc(50%+0.4px)]" data-name="Vector">
                <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 9.33333 5.33333">
                  <path d={svgPaths.p2fc97600} fill="var(--fill-0, #4D4E53)" id="Vector" />
                </svg>
              </div>
            </div>
          </div>
        </div>
      </div>
      <div className="relative shrink-0 w-full" data-name=".Interactive cell">
        <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
        <div className="flex flex-row items-center size-full">
          <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative w-full">
            <Label45 />
            <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / chevron-down">
              <div className="-translate-y-1/2 absolute aspect-[14/8] left-[23.32%] right-[18.35%] top-[calc(50%+0.4px)]" data-name="Vector">
                <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 9.33333 5.33333">
                  <path d={svgPaths.p2fc97600} fill="var(--fill-0, #4D4E53)" id="Vector" />
                </svg>
              </div>
            </div>
          </div>
        </div>
      </div>
      <div className="relative shrink-0 w-full" data-name=".Interactive cell">
        <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
        <div className="flex flex-row items-center size-full">
          <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative w-full">
            <Label46 />
            <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / chevron-down">
              <div className="-translate-y-1/2 absolute aspect-[14/8] left-[23.32%] right-[18.35%] top-[calc(50%+0.4px)]" data-name="Vector">
                <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 9.33333 5.33333">
                  <path d={svgPaths.p2fc97600} fill="var(--fill-0, #4D4E53)" id="Vector" />
                </svg>
              </div>
            </div>
          </div>
        </div>
      </div>
      <div className="relative shrink-0 w-full" data-name=".Interactive cell">
        <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
        <div className="flex flex-row items-center size-full">
          <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative w-full">
            <Label47 />
            <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / chevron-down">
              <div className="-translate-y-1/2 absolute aspect-[14/8] left-[23.32%] right-[18.35%] top-[calc(50%+0.4px)]" data-name="Vector">
                <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 9.33333 5.33333">
                  <path d={svgPaths.p2fc97600} fill="var(--fill-0, #4D4E53)" id="Vector" />
                </svg>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function Header11() {
  return (
    <div className="content-stretch flex gap-[4px] items-center relative shrink-0" data-name="Header">
      <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / circle-info">
        <div className="absolute inset-[10%]" data-name="Vector">
          <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.8 12.8">
            <path d={svgPaths.p18411100} fill="var(--fill-0, #4D4E53)" id="Vector" />
          </svg>
        </div>
      </div>
      <div className="flex flex-col font-['Inter:Bold',sans-serif] font-bold justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#0b0b0b] text-[14px] text-ellipsis whitespace-nowrap">
        <p className="leading-[20px] overflow-hidden">{`Available Date & Time `}</p>
      </div>
    </div>
  );
}

function Left11() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[8px] items-center min-h-px min-w-px relative" data-name="Left">
      <Header11 />
    </div>
  );
}

function Label48() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">Jan 27, 2026, 10:45 PM EST</p>
      </div>
    </div>
  );
}

function Label49() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">Set date and time</p>
      </div>
    </div>
  );
}

function Label50() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">Set date and time</p>
      </div>
    </div>
  );
}

function Label51() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">Set date and time</p>
      </div>
    </div>
  );
}

function Label52() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">Set date and time</p>
      </div>
    </div>
  );
}

function Label53() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">Set date and time</p>
      </div>
    </div>
  );
}

function Header12() {
  return (
    <div className="content-stretch flex gap-[4px] items-center relative shrink-0" data-name="Header">
      <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / circle-info">
        <div className="absolute inset-[10%]" data-name="Vector">
          <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.8 12.8">
            <path d={svgPaths.p18411100} fill="var(--fill-0, #4D4E53)" id="Vector" />
          </svg>
        </div>
      </div>
      <div className="flex flex-col font-['Inter:Bold',sans-serif] font-bold justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#0b0b0b] text-[14px] text-ellipsis whitespace-nowrap">
        <p className="leading-[20px] overflow-hidden">{`Reimbursement `}</p>
      </div>
    </div>
  );
}

function Left12() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[8px] items-center min-h-px min-w-px relative" data-name="Left">
      <Header12 />
    </div>
  );
}

function Label54() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">$0.00</p>
      </div>
    </div>
  );
}

function Label55() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">$0.00</p>
      </div>
    </div>
  );
}

function Label56() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">$0.00</p>
      </div>
    </div>
  );
}

function Label57() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">$0.00</p>
      </div>
    </div>
  );
}

function Label58() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">$0.00</p>
      </div>
    </div>
  );
}

function Label59() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">$0.00</p>
      </div>
    </div>
  );
}

function Header13() {
  return (
    <div className="content-stretch flex gap-[4px] items-center relative shrink-0" data-name="Header">
      <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / circle-info">
        <div className="absolute inset-[10%]" data-name="Vector">
          <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.8 12.8">
            <path d={svgPaths.p18411100} fill="var(--fill-0, #4D4E53)" id="Vector" />
          </svg>
        </div>
      </div>
      <div className="flex flex-col font-['Inter:Bold',sans-serif] font-bold justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#0b0b0b] text-[14px] text-ellipsis whitespace-nowrap">
        <p className="leading-[20px] overflow-hidden">Net Buyer Cost</p>
      </div>
    </div>
  );
}

function Left13() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[8px] items-center min-h-px min-w-px relative" data-name="Left">
      <Header13 />
    </div>
  );
}

function Label60() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">$100.00</p>
      </div>
    </div>
  );
}

function Label61() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">$100.00</p>
      </div>
    </div>
  );
}

function Label62() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">$100.00</p>
      </div>
    </div>
  );
}

function Label63() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">$100.00</p>
      </div>
    </div>
  );
}

function Label64() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">$100.00</p>
      </div>
    </div>
  );
}

function Label65() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">$100.00</p>
      </div>
    </div>
  );
}

function DataTable() {
  return (
    <div className="bg-white relative rounded-[8px] shrink-0" data-name="Data table">
      <div className="content-stretch flex items-start overflow-clip relative rounded-[inherit]">
        <Column />
        <div className="bg-white content-stretch flex flex-col items-start relative self-stretch shrink-0 w-[180px]" data-name="Column">
          <div className="bg-[#eaeaea] h-[48px] relative shrink-0 w-full" data-name=".Header">
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center p-[16px] relative size-full">
                <Left2 />
                <div className="-translate-y-1/2 absolute bg-[rgba(24,29,31,0.15)] content-stretch flex h-[14px] items-center overflow-clip right-0 top-1/2 w-[2px]" data-name="Pipe - R">
                  <div className="bg-[rgba(24,29,31,0.15)] h-full shrink-0 w-[2px]" data-name="shape" />
                </div>
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <Label />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <Label1 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <Label2 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <Label3 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <Label4 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <Label5 />
              </div>
            </div>
          </div>
        </div>
        <div className="bg-white content-stretch flex flex-col items-start relative self-stretch shrink-0 w-[190px]" data-name="Column">
          <div className="bg-[#eaeaea] h-[48px] relative shrink-0 w-full" data-name=".Header">
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center p-[16px] relative size-full">
                <Left3 />
                <div className="-translate-y-1/2 absolute bg-[rgba(24,29,31,0.15)] content-stretch flex h-[14px] items-center left-0 overflow-clip top-1/2 w-[2px]" data-name="Pipe - L">
                  <div className="bg-[rgba(24,29,31,0.15)] h-full shrink-0 w-[2px]" data-name="shape" />
                </div>
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / ellipsis-vertical">
                  <div className="absolute inset-[15%_41.25%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 2.8 11.2">
                      <path d={svgPaths.p1ec51580} fill="var(--fill-0, #4D4E53)" id="Vector" />
                    </svg>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Interactive cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative w-full">
                <Label6 />
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / pen-to-square">
                  <div className="absolute inset-[10.82%_10.82%_10%_10%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.6681 12.6681">
                      <path d={svgPaths.p3c006800} fill="var(--fill-0, #4D4E53)" id="Vector" />
                    </svg>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Interactive cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative w-full">
                <Label7 />
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / pen-to-square">
                  <div className="absolute inset-[10.82%_10.82%_10%_10%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.6681 12.6681">
                      <path d={svgPaths.p3c006800} fill="var(--fill-0, #4D4E53)" id="Vector" />
                    </svg>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Interactive cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative w-full">
                <Label8 />
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / pen-to-square">
                  <div className="absolute inset-[10.82%_10.82%_10%_10%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.6681 12.6681">
                      <path d={svgPaths.p3c006800} fill="var(--fill-0, #4D4E53)" id="Vector" />
                    </svg>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Interactive cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative w-full">
                <Label9 />
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / pen-to-square">
                  <div className="absolute inset-[10.82%_10.82%_10%_10%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.6681 12.6681">
                      <path d={svgPaths.p3c006800} fill="var(--fill-0, #4D4E53)" id="Vector" />
                    </svg>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Interactive cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative w-full">
                <Label10 />
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / pen-to-square">
                  <div className="absolute inset-[10.82%_10.82%_10%_10%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.6681 12.6681">
                      <path d={svgPaths.p3c006800} fill="var(--fill-0, #4D4E53)" id="Vector" />
                    </svg>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Interactive cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative w-full">
                <Label11 />
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / pen-to-square">
                  <div className="absolute inset-[10.82%_10.82%_10%_10%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.6681 12.6681">
                      <path d={svgPaths.p3c006800} fill="var(--fill-0, #4D4E53)" id="Vector" />
                    </svg>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
        <Column1 />
        <Column2 />
        <div className="bg-white content-stretch flex flex-col items-start relative self-stretch shrink-0 w-[310px]" data-name="Column">
          <div className="bg-[#eaeaea] h-[48px] relative shrink-0 w-full" data-name=".Header">
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center p-[16px] relative size-full">
                <Left6 />
                <div className="-translate-y-1/2 absolute bg-[rgba(24,29,31,0.15)] content-stretch flex h-[14px] items-center overflow-clip right-0 top-1/2 w-[2px]" data-name="Pipe - R">
                  <div className="bg-[rgba(24,29,31,0.15)] h-full shrink-0 w-[2px]" data-name="shape" />
                </div>
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <Label18 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <Label19 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <Label20 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <Label21 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <Label22 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <Label23 />
              </div>
            </div>
          </div>
        </div>
        <div className="bg-white content-stretch flex flex-col items-start relative self-stretch shrink-0 w-[113px]" data-name="Column">
          <div className="bg-[#eaeaea] h-[48px] relative shrink-0 w-full" data-name=".Header">
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center p-[16px] relative size-full">
                <Left7 />
                <div className="-translate-y-1/2 absolute bg-[rgba(24,29,31,0.15)] content-stretch flex h-[14px] items-center overflow-clip right-0 top-1/2 w-[2px]" data-name="Pipe - R">
                  <div className="bg-[rgba(24,29,31,0.15)] h-full shrink-0 w-[2px]" data-name="shape" />
                </div>
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <Label24 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <Label25 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <Label26 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <Label27 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <Label28 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <Label29 />
              </div>
            </div>
          </div>
        </div>
        <div className="bg-white content-stretch flex flex-col items-start relative self-stretch shrink-0 w-[186px]" data-name="Column">
          <div className="bg-[#eaeaea] h-[48px] relative shrink-0 w-full" data-name=".Header">
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center p-[16px] relative size-full">
                <Left8 />
                <div className="-translate-y-1/2 absolute bg-[rgba(24,29,31,0.15)] content-stretch flex h-[14px] items-center left-0 overflow-clip top-1/2 w-[2px]" data-name="Pipe - L">
                  <div className="bg-[rgba(24,29,31,0.15)] h-full shrink-0 w-[2px]" data-name="shape" />
                </div>
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Interactive cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative w-full">
                <Label30 />
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / pen-to-square">
                  <div className="absolute inset-[10.82%_10.82%_10%_10%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.6681 12.6681">
                      <path d={svgPaths.p3c006800} fill="var(--fill-0, #4D4E53)" id="Vector" />
                    </svg>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Interactive cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative w-full">
                <Label31 />
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / pen-to-square">
                  <div className="absolute inset-[10.82%_10.82%_10%_10%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.6681 12.6681">
                      <path d={svgPaths.p3c006800} fill="var(--fill-0, #4D4E53)" id="Vector" />
                    </svg>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Interactive cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative w-full">
                <Label32 />
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / pen-to-square">
                  <div className="absolute inset-[10.82%_10.82%_10%_10%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.6681 12.6681">
                      <path d={svgPaths.p3c006800} fill="var(--fill-0, #4D4E53)" id="Vector" />
                    </svg>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Interactive cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative w-full">
                <Label33 />
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / pen-to-square">
                  <div className="absolute inset-[10.82%_10.82%_10%_10%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.6681 12.6681">
                      <path d={svgPaths.p3c006800} fill="var(--fill-0, #4D4E53)" id="Vector" />
                    </svg>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Interactive cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative w-full">
                <Label34 />
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / pen-to-square">
                  <div className="absolute inset-[10.82%_10.82%_10%_10%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.6681 12.6681">
                      <path d={svgPaths.p3c006800} fill="var(--fill-0, #4D4E53)" id="Vector" />
                    </svg>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Interactive cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative w-full">
                <Label35 />
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / pen-to-square">
                  <div className="absolute inset-[10.82%_10.82%_10%_10%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.6681 12.6681">
                      <path d={svgPaths.p3c006800} fill="var(--fill-0, #4D4E53)" id="Vector" />
                    </svg>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
        <div className="bg-white content-stretch flex flex-col items-start relative self-stretch shrink-0 w-[186px]" data-name="Column">
          <div className="bg-[#eaeaea] h-[48px] relative shrink-0 w-full" data-name=".Header">
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center p-[16px] relative size-full">
                <Left9 />
                <div className="-translate-y-1/2 absolute bg-[rgba(24,29,31,0.15)] content-stretch flex h-[14px] items-center left-0 overflow-clip top-1/2 w-[2px]" data-name="Pipe - L">
                  <div className="bg-[rgba(24,29,31,0.15)] h-full shrink-0 w-[2px]" data-name="shape" />
                </div>
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Interactive cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative w-full">
                <Label36 />
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / pen-to-square">
                  <div className="absolute inset-[10.82%_10.82%_10%_10%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.6681 12.6681">
                      <path d={svgPaths.p3c006800} fill="var(--fill-0, #4D4E53)" id="Vector" />
                    </svg>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Interactive cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative w-full">
                <Label37 />
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / pen-to-square">
                  <div className="absolute inset-[10.82%_10.82%_10%_10%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.6681 12.6681">
                      <path d={svgPaths.p3c006800} fill="var(--fill-0, #4D4E53)" id="Vector" />
                    </svg>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Interactive cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative w-full">
                <Label38 />
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / pen-to-square">
                  <div className="absolute inset-[10.82%_10.82%_10%_10%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.6681 12.6681">
                      <path d={svgPaths.p3c006800} fill="var(--fill-0, #4D4E53)" id="Vector" />
                    </svg>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Interactive cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative w-full">
                <Label39 />
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / pen-to-square">
                  <div className="absolute inset-[10.82%_10.82%_10%_10%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.6681 12.6681">
                      <path d={svgPaths.p3c006800} fill="var(--fill-0, #4D4E53)" id="Vector" />
                    </svg>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Interactive cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative w-full">
                <Label40 />
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / pen-to-square">
                  <div className="absolute inset-[10.82%_10.82%_10%_10%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.6681 12.6681">
                      <path d={svgPaths.p3c006800} fill="var(--fill-0, #4D4E53)" id="Vector" />
                    </svg>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Interactive cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative w-full">
                <Label41 />
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / pen-to-square">
                  <div className="absolute inset-[10.82%_10.82%_10%_10%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.6681 12.6681">
                      <path d={svgPaths.p3c006800} fill="var(--fill-0, #4D4E53)" id="Vector" />
                    </svg>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
        <Column3 />
        <div className="bg-white content-stretch flex flex-col items-start relative self-stretch shrink-0 w-[291px]" data-name="Column">
          <div className="bg-[#eaeaea] h-[48px] relative shrink-0 w-full" data-name=".Header">
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center p-[16px] relative size-full">
                <Left11 />
                <div className="-translate-y-1/2 absolute bg-[rgba(24,29,31,0.15)] content-stretch flex h-[14px] items-center left-0 overflow-clip top-1/2 w-[2px]" data-name="Pipe - L">
                  <div className="bg-[rgba(24,29,31,0.15)] h-full shrink-0 w-[2px]" data-name="shape" />
                </div>
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / ellipsis-vertical">
                  <div className="absolute inset-[15%_41.25%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 2.8 11.2">
                      <path d={svgPaths.p1ec51580} fill="var(--fill-0, #4D4E53)" id="Vector" />
                    </svg>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Interactive cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative w-full">
                <Label48 />
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / pen-to-square">
                  <div className="absolute inset-[10.82%_10.82%_10%_10%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.6681 12.6681">
                      <path d={svgPaths.p3c006800} fill="var(--fill-0, #4D4E53)" id="Vector" />
                    </svg>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Interactive cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative w-full">
                <Label49 />
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / pen-to-square">
                  <div className="absolute inset-[10.82%_10.82%_10%_10%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.6681 12.6681">
                      <path d={svgPaths.p3c006800} fill="var(--fill-0, #4D4E53)" id="Vector" />
                    </svg>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Interactive cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative w-full">
                <Label50 />
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / pen-to-square">
                  <div className="absolute inset-[10.82%_10.82%_10%_10%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.6681 12.6681">
                      <path d={svgPaths.p3c006800} fill="var(--fill-0, #4D4E53)" id="Vector" />
                    </svg>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Interactive cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative w-full">
                <Label51 />
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / pen-to-square">
                  <div className="absolute inset-[10.82%_10.82%_10%_10%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.6681 12.6681">
                      <path d={svgPaths.p3c006800} fill="var(--fill-0, #4D4E53)" id="Vector" />
                    </svg>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Interactive cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative w-full">
                <Label52 />
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / pen-to-square">
                  <div className="absolute inset-[10.82%_10.82%_10%_10%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.6681 12.6681">
                      <path d={svgPaths.p3c006800} fill="var(--fill-0, #4D4E53)" id="Vector" />
                    </svg>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Interactive cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative w-full">
                <Label53 />
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / pen-to-square">
                  <div className="absolute inset-[10.82%_10.82%_10%_10%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.6681 12.6681">
                      <path d={svgPaths.p3c006800} fill="var(--fill-0, #4D4E53)" id="Vector" />
                    </svg>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
        <div className="bg-white content-stretch flex flex-col items-start relative self-stretch shrink-0 w-[148px]" data-name="Column">
          <div className="bg-[#eaeaea] h-[48px] relative shrink-0 w-full" data-name=".Header">
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center p-[16px] relative size-full">
                <Left12 />
                <div className="-translate-y-1/2 absolute bg-[rgba(24,29,31,0.15)] content-stretch flex h-[14px] items-center overflow-clip right-0 top-1/2 w-[2px]" data-name="Pipe - R">
                  <div className="bg-[rgba(24,29,31,0.15)] h-full shrink-0 w-[2px]" data-name="shape" />
                </div>
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <Label54 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <Label55 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <Label56 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <Label57 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <Label58 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <Label59 />
              </div>
            </div>
          </div>
        </div>
        <div className="bg-white content-stretch flex flex-col items-start relative self-stretch shrink-0 w-[216px]" data-name="Column">
          <div className="bg-[#eaeaea] h-[48px] relative shrink-0 w-full" data-name=".Header">
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center p-[16px] relative size-full">
                <Left13 />
                <div className="-translate-y-1/2 absolute bg-[rgba(24,29,31,0.15)] content-stretch flex h-[14px] items-center left-0 overflow-clip top-1/2 w-[2px]" data-name="Pipe - L">
                  <div className="bg-[rgba(24,29,31,0.15)] h-full shrink-0 w-[2px]" data-name="shape" />
                </div>
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / ellipsis-vertical">
                  <div className="absolute inset-[15%_41.25%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 2.8 11.2">
                      <path d={svgPaths.p1ec51580} fill="var(--fill-0, #4D4E53)" id="Vector" />
                    </svg>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Interactive cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative w-full">
                <Label60 />
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / pen-to-square">
                  <div className="absolute inset-[10.82%_10.82%_10%_10%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.6681 12.6681">
                      <path d={svgPaths.p3c006800} fill="var(--fill-0, #4D4E53)" id="Vector" />
                    </svg>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Interactive cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative w-full">
                <Label61 />
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / pen-to-square">
                  <div className="absolute inset-[10.82%_10.82%_10%_10%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.6681 12.6681">
                      <path d={svgPaths.p3c006800} fill="var(--fill-0, #4D4E53)" id="Vector" />
                    </svg>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Interactive cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative w-full">
                <Label62 />
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / pen-to-square">
                  <div className="absolute inset-[10.82%_10.82%_10%_10%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.6681 12.6681">
                      <path d={svgPaths.p3c006800} fill="var(--fill-0, #4D4E53)" id="Vector" />
                    </svg>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Interactive cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative w-full">
                <Label63 />
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / pen-to-square">
                  <div className="absolute inset-[10.82%_10.82%_10%_10%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.6681 12.6681">
                      <path d={svgPaths.p3c006800} fill="var(--fill-0, #4D4E53)" id="Vector" />
                    </svg>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Interactive cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative w-full">
                <Label64 />
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / pen-to-square">
                  <div className="absolute inset-[10.82%_10.82%_10%_10%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.6681 12.6681">
                      <path d={svgPaths.p3c006800} fill="var(--fill-0, #4D4E53)" id="Vector" />
                    </svg>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Interactive cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative w-full">
                <Label65 />
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / pen-to-square">
                  <div className="absolute inset-[10.82%_10.82%_10%_10%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.6681 12.6681">
                      <path d={svgPaths.p3c006800} fill="var(--fill-0, #4D4E53)" id="Vector" />
                    </svg>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
      <div aria-hidden="true" className="absolute border border-[#dcdcdc] border-solid inset-0 pointer-events-none rounded-[8px]" />
    </div>
  );
}

function Frame13() {
  return (
    <div className="content-stretch flex flex-col gap-[12px] items-start py-[16px] relative shrink-0 w-full">
      <div aria-hidden="true" className="absolute border-[#eaeaea] border-b border-solid inset-0 pointer-events-none" />
      <div className="bg-white h-[32px] relative rounded-[8px] shrink-0 w-full" data-name="OButton">
        <div className="flex flex-row items-center justify-center size-full">
          <div className="content-stretch flex gap-[4px] items-center justify-center px-[16px] py-[8px] relative size-full">
            <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / truck-fast">
              <div className="absolute inset-[15%_0_10%_0]" data-name="Vector">
                <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 12">
                  <path d={svgPaths.p19d93980} fill="var(--fill-0, #003C71)" id="Vector" />
                </svg>
              </div>
            </div>
            <p className="font-['Inter:Bold',sans-serif] font-bold leading-[20px] not-italic relative shrink-0 text-[#003c71] text-[14px] whitespace-nowrap">{`Use delivery now `}</p>
          </div>
        </div>
      </div>
      <div className="bg-white h-[32px] relative rounded-[8px] shrink-0 w-full" data-name="OButton">
        <div className="flex flex-row items-center justify-center size-full">
          <div className="content-stretch flex gap-[4px] items-center justify-center px-[16px] py-[8px] relative size-full">
            <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / scroll">
              <div className="absolute inset-[15%_10%_15%_5%]" data-name="Vector">
                <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 13.6 11.2">
                  <path d={svgPaths.p1def6d00} fill="var(--fill-0, #003C71)" id="Vector" />
                </svg>
              </div>
            </div>
            <p className="font-['Inter:Bold',sans-serif] font-bold leading-[20px] not-italic relative shrink-0 text-[#003c71] text-[14px] whitespace-nowrap">Create DMS quote</p>
          </div>
        </div>
      </div>
      <div className="bg-white h-[32px] relative rounded-[8px] shrink-0 w-full" data-name="OButton">
        <div className="flex flex-row items-center justify-center size-full">
          <div className="content-stretch flex gap-[4px] items-center justify-center px-[16px] py-[8px] relative size-full">
            <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / floppy-disk">
              <div className="absolute inset-[15%]" data-name="Vector">
                <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 11.2 11.2">
                  <path d={svgPaths.p2c972380} fill="var(--fill-0, #003C71)" id="Vector" />
                </svg>
              </div>
            </div>
            <p className="font-['Inter:Bold',sans-serif] font-bold leading-[20px] not-italic relative shrink-0 text-[#003c71] text-[14px] whitespace-nowrap">Save changes</p>
          </div>
        </div>
      </div>
    </div>
  );
}

function Frame16() {
  return (
    <div className="content-stretch flex flex-col gap-[16px] items-start pb-[16px] pt-[4px] px-[16px] relative rounded-[8px] shrink-0 w-[205px]">
      <div aria-hidden="true" className="absolute border border-[#eaeaea] border-solid inset-0 pointer-events-none rounded-[8px]" />
      <Frame13 />
      <div className="bg-white h-[32px] relative rounded-[8px] shrink-0 w-full" data-name="OButton">
        <div className="flex flex-row items-center justify-center size-full">
          <div className="content-stretch flex gap-[4px] items-center justify-center px-[16px] py-[4px] relative size-full">
            <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / circle-xmark">
              <div className="absolute inset-[10%]" data-name="Vector">
                <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.8 12.8">
                  <path d={svgPaths.p34e9b80} fill="var(--fill-0, #C23F2E)" id="Vector" />
                </svg>
              </div>
            </div>
            <p className="font-['Inter:Bold',sans-serif] font-bold leading-[20px] not-italic relative shrink-0 text-[#c23f2e] text-[14px] whitespace-nowrap">Cancel order</p>
          </div>
        </div>
      </div>
    </div>
  );
}

function Frame8() {
  return (
    <div className="content-stretch flex font-['Inter:Regular',sans-serif] font-normal items-center justify-between leading-[20px] not-italic relative shrink-0 text-[14px] w-full whitespace-nowrap">
      <p className="relative shrink-0 text-[#656565]">Subtotal:</p>
      <p className="relative shrink-0 text-[#202020]">$400.00</p>
    </div>
  );
}

function Frame9() {
  return (
    <div className="content-stretch flex font-['Inter:Regular',sans-serif] font-normal items-center justify-between leading-[20px] not-italic relative shrink-0 text-[14px] w-full whitespace-nowrap">
      <p className="relative shrink-0 text-[#656565]">Total Core Price:</p>
      <p className="relative shrink-0 text-[#202020]">$0</p>
    </div>
  );
}

function Frame10() {
  return (
    <div className="content-stretch flex font-['Inter:Regular',sans-serif] font-normal items-center justify-between leading-[20px] not-italic relative shrink-0 text-[14px] w-full whitespace-nowrap">
      <p className="relative shrink-0 text-[#656565]">Taxes/Fees:</p>
      <p className="relative shrink-0 text-[#202020]">$6.00</p>
    </div>
  );
}

function Frame6() {
  return (
    <div className="content-stretch flex gap-[4px] items-center relative shrink-0">
      <p className="font-['Inter:Regular',sans-serif] font-normal leading-[20px] not-italic relative shrink-0 text-[#656565] text-[14px] whitespace-nowrap">Transaction Fee:</p>
      <div className="overflow-clip relative shrink-0 size-[20px]" data-name="Icon / circle-info">
        <div className="absolute inset-[10%]" data-name="Vector">
          <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
            <path d={svgPaths.p11e2bd00} fill="var(--fill-0, #4D4E53)" id="Vector" />
          </svg>
        </div>
      </div>
    </div>
  );
}

function Frame11() {
  return (
    <div className="content-stretch flex items-center justify-between relative shrink-0 w-full">
      <Frame6 />
      <p className="font-['Inter:Regular',sans-serif] font-normal leading-[20px] not-italic relative shrink-0 text-[#202020] text-[14px] whitespace-nowrap">$3.00</p>
    </div>
  );
}

function Frame7() {
  return (
    <div className="content-stretch flex gap-[4px] items-center relative shrink-0">
      <p className="font-['Inter:Regular',sans-serif] font-normal leading-[20px] not-italic relative shrink-0 text-[#656565] text-[14px] whitespace-nowrap">Surcharge Fee:</p>
      <div className="overflow-clip relative shrink-0 size-[20px]" data-name="Icon / circle-info">
        <div className="absolute inset-[10%]" data-name="Vector">
          <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 16 16">
            <path d={svgPaths.p11e2bd00} fill="var(--fill-0, #4D4E53)" id="Vector" />
          </svg>
        </div>
      </div>
    </div>
  );
}

function Frame12() {
  return (
    <div className="content-stretch flex items-center justify-between relative shrink-0 w-full">
      <Frame7 />
      <p className="font-['Inter:Regular',sans-serif] font-normal leading-[20px] not-italic relative shrink-0 text-[#202020] text-[14px] whitespace-nowrap">$2.00</p>
    </div>
  );
}

function Frame14() {
  return (
    <div className="content-stretch flex flex-col gap-[12px] items-start py-[16px] relative shrink-0 w-full">
      <div aria-hidden="true" className="absolute border-[#eaeaea] border-b border-solid inset-0 pointer-events-none" />
      <Frame8 />
      <Frame9 />
      <Frame10 />
      <Frame11 />
      <Frame12 />
    </div>
  );
}

function Frame20() {
  return (
    <div className="content-stretch flex font-['Inter:Bold',sans-serif] font-bold items-center justify-between leading-[0] relative shrink-0 text-[#202020] text-[20px] w-full">
      <div className="flex flex-col justify-center relative shrink-0">
        <p className="leading-[28px]">Total:</p>
      </div>
      <div className="flex flex-col justify-center relative shrink-0">
        <p className="leading-[28px]">$417.00</p>
      </div>
    </div>
  );
}

function Frame21() {
  return (
    <div className="content-stretch flex font-['Inter:Regular',sans-serif] font-normal items-center justify-between leading-[20px] relative shrink-0 text-[#3a6f3b] text-[14px] w-full">
      <p className="relative shrink-0">Total Reimbursement:</p>
      <p className="relative shrink-0">$0.00</p>
    </div>
  );
}

function Frame17() {
  return (
    <div className="content-stretch flex flex-col gap-[4px] items-start not-italic relative shrink-0 w-full whitespace-nowrap">
      <Frame20 />
      <Frame21 />
    </div>
  );
}

function Frame22() {
  return (
    <div className="content-stretch flex gap-[17px] items-center relative shrink-0 w-full">
      <div className="bg-white flex-[1_0_0] min-h-px min-w-px relative rounded-[8px]" data-name="OButton">
        <div aria-hidden="true" className="absolute border border-[#003c71] border-solid inset-0 pointer-events-none rounded-[8px]" />
        <div className="flex flex-row items-center justify-center size-full">
          <div className="content-stretch flex items-center justify-center px-[16px] py-[8px] relative w-full">
            <p className="font-['Inter:Bold',sans-serif] font-bold leading-[24px] not-italic relative shrink-0 text-[#003c71] text-[16px] whitespace-nowrap">Process Payment</p>
          </div>
        </div>
      </div>
      <div className="bg-white flex-[1_0_0] min-h-px min-w-px relative rounded-[8px]" data-name="OButton">
        <div aria-hidden="true" className="absolute border border-[#003c71] border-solid inset-0 pointer-events-none rounded-[8px]" />
        <div className="flex flex-row items-center justify-center size-full">
          <div className="content-stretch flex items-center justify-center px-[16px] py-[8px] relative w-full">
            <p className="font-['Inter:Bold',sans-serif] font-bold leading-[24px] not-italic relative shrink-0 text-[#003c71] text-[16px] whitespace-nowrap">Payment Details</p>
          </div>
        </div>
      </div>
    </div>
  );
}

function Frame15() {
  return (
    <div className="content-stretch flex flex-col gap-[16px] items-start pb-[16px] pt-[4px] px-[16px] relative rounded-[8px] shrink-0 w-[367px]">
      <div aria-hidden="true" className="absolute border border-[#eaeaea] border-solid inset-0 pointer-events-none rounded-[8px]" />
      <Frame14 />
      <Frame17 />
      <Frame22 />
      <div className="bg-[#003c71] relative rounded-[8px] shrink-0 w-full" data-name="OButton">
        <div className="flex flex-row items-center justify-center size-full">
          <div className="content-stretch flex items-center justify-center px-[16px] py-[8px] relative w-full">
            <p className="font-['Inter:Bold',sans-serif] font-bold leading-[24px] not-italic relative shrink-0 text-[16px] text-white whitespace-nowrap">Submit</p>
          </div>
        </div>
      </div>
    </div>
  );
}

function Frame18() {
  return (
    <div className="content-stretch flex gap-[16px] items-start relative shrink-0">
      <Frame16 />
      <Frame15 />
    </div>
  );
}

function Frame19() {
  return (
    <div className="content-stretch flex flex-col gap-[16px] items-end relative shrink-0">
      <DataTable />
      <Frame18 />
    </div>
  );
}

function Container() {
  return (
    <div className="absolute content-stretch flex flex-col gap-[16px] items-start left-[204px] top-[181px] w-[2453px]" data-name="Container">
      <div className="bg-white relative shrink-0 w-full" data-name="Modal Elemets">
        <div aria-hidden="true" className="absolute border-[#eaeaea] border-b border-solid inset-0 pointer-events-none" />
        <div className="flex flex-row items-center size-full">
          <div className="content-stretch flex items-center justify-between px-[16px] py-[12px] relative w-full">
            <LeftSection />
            <Frame24 />
          </div>
        </div>
      </div>
      <ProductDetails />
      <Frame />
      <Frame19 />
    </div>
  );
}

export default function Orders() {
  return (
    <div className="bg-white relative size-full" data-name="Orders">
      <Header />
      <Sidebar />
      <Container />
    </div>
  );
}