import svgPaths from "./svg-4j9tucpt3w";
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
    <div className="absolute content-stretch flex flex-col items-start left-0 top-0 w-[2371px]" data-name="Header">
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
            <path d={svgPaths.p2997f700} fill="var(--fill-0, #4D4E53)" id="Vector" />
          </svg>
        </div>
      </div>
      <p className="font-['Inter:Bold',sans-serif] font-bold leading-[24px] not-italic relative shrink-0 text-[#4d4e53] text-[16px] text-center whitespace-nowrap">Overview</p>
    </div>
  );
}

function Text1() {
  return (
    <div className="content-stretch flex gap-[4px] items-center justify-center px-[12px] relative shrink-0" data-name="Text">
      <div className="overflow-clip relative shrink-0 size-[24px]" data-name="Checkmark">
        <div className="absolute inset-[20%_10%]" data-name="Vector">
          <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 19.2 14.4">
            <path d={svgPaths.p3ec28d00} fill="var(--fill-0, #003C71)" id="Vector" />
          </svg>
        </div>
      </div>
      <p className="font-['Inter:Bold',sans-serif] font-bold leading-[24px] not-italic relative shrink-0 text-[#003c71] text-[16px] text-center whitespace-nowrap">Transaction</p>
    </div>
  );
}

function Frame3() {
  return (
    <div className="content-stretch flex items-start justify-between relative shrink-0 w-full">
      <div className="content-stretch flex items-center relative rounded-br-[8px] rounded-tr-[8px] shrink-0" data-name="Tab">
        <div className="bg-white content-stretch flex flex-col gap-[8px] items-start pt-[8px] relative shrink-0" data-name="Tab Elements">
          <Text />
          <div className="bg-[#bfbfbf] h-[2px] shrink-0 w-full" data-name="Underline" />
        </div>
        <div className="bg-white content-stretch flex flex-col gap-[8px] items-start pt-[8px] relative shrink-0" data-name="Tab Elements">
          <Text1 />
          <div className="bg-[#003c71] h-[2px] rounded-[999px] shrink-0 w-full" data-name="Underline" />
        </div>
      </div>
      <div className="bg-white content-stretch flex gap-[4px] items-center justify-center px-[16px] py-[8px] relative rounded-[8px] shrink-0" data-name="OButton">
        <div className="overflow-clip relative shrink-0 size-[24px]" data-name="Icon / gear">
          <div className="absolute inset-[7.5%_10.13%_7.5%_10.24%]" data-name="Vector">
            <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 19.1113 20.4">
              <path d={svgPaths.p234db500} fill="var(--fill-0, #003C71)" id="Vector" />
            </svg>
          </div>
        </div>
        <p className="font-['Inter:Bold',sans-serif] font-bold leading-[24px] not-italic relative shrink-0 text-[#003c71] text-[16px] whitespace-nowrap">Customize page</p>
      </div>
    </div>
  );
}

function Counter() {
  return (
    <div className="absolute bg-[#16233d] content-stretch flex items-center justify-center left-[79px] px-[6px] py-[4px] rounded-[100px] top-[-8px] w-[26px]" data-name="Counter">
      <div aria-hidden="true" className="absolute border-2 border-solid border-white inset-0 pointer-events-none rounded-[100px]" />
      <div className="flex flex-col font-['Arial:Bold',sans-serif] justify-center leading-[0] not-italic relative shrink-0 text-[12px] text-right text-white tracking-[0.25px] uppercase whitespace-nowrap">
        <p className="leading-[16px]">3</p>
      </div>
    </div>
  );
}

function Frame2() {
  return (
    <div className="content-stretch flex gap-[16px] items-center relative shrink-0">
      <div className="bg-white content-stretch flex gap-[8px] items-center justify-center px-[12px] py-[8px] relative rounded-[100px] shrink-0" data-name="Filter tag">
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
        <Counter />
      </div>
      <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[8px] relative rounded-[8px] shrink-0 w-[300px]" data-name="Search field">
        <div aria-hidden="true" className="absolute border border-[#bfbfbf] border-solid inset-0 pointer-events-none rounded-[8px]" />
        <div className="overflow-clip relative shrink-0 size-[24px]" data-name="Search Icon">
          <div className="absolute inset-[10%_10%_9.99%_10%]" data-name="Vector">
            <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 19.1991 19.2028">
              <path d={svgPaths.p30863d80} fill="var(--fill-0, #4D4E53)" id="Vector" />
            </svg>
          </div>
        </div>
        <p className="font-['Inter:Regular',sans-serif] font-normal leading-[24px] not-italic relative shrink-0 text-[#656565] text-[16px] whitespace-nowrap">Search activity or analytics</p>
      </div>
    </div>
  );
}

function Frame1() {
  return (
    <div className="content-stretch flex items-center relative shrink-0">
      <div className="bg-white content-stretch flex gap-[4px] items-center justify-center px-[16px] py-[8px] relative rounded-[8px] shrink-0" data-name="OButton">
        <div className="overflow-clip relative shrink-0 size-[24px]" data-name="Icon / box-archive">
          <div className="absolute inset-[15%_10%]" data-name="Vector">
            <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 19.2 16.8">
              <path d={svgPaths.p11da2700} fill="var(--fill-0, #003C71)" id="Vector" />
            </svg>
          </div>
        </div>
        <p className="font-['Inter:Bold',sans-serif] font-bold leading-[24px] not-italic relative shrink-0 text-[#003c71] text-[16px] whitespace-nowrap">Archive</p>
      </div>
      <div className="bg-white content-stretch flex gap-[4px] items-center justify-center px-[16px] py-[8px] relative rounded-[8px] shrink-0" data-name="OButton">
        <div className="overflow-clip relative shrink-0 size-[24px]" data-name="Icon / print">
          <div className="absolute inset-[10%]" data-name="Vector">
            <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 19.2 19.2">
              <path d={svgPaths.p3d2d6400} fill="var(--fill-0, #003C71)" id="Vector" />
            </svg>
          </div>
        </div>
        <p className="font-['Inter:Bold',sans-serif] font-bold leading-[24px] not-italic relative shrink-0 text-[#003c71] text-[16px] whitespace-nowrap">Print</p>
      </div>
    </div>
  );
}

function Frame() {
  return (
    <div className="content-stretch flex items-end justify-between relative shrink-0 w-full">
      <Frame2 />
      <Frame1 />
    </div>
  );
}

function Header1() {
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
        <p className="leading-[20px] overflow-hidden">Transaction #</p>
      </div>
    </div>
  );
}

function Left() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[8px] items-center min-h-px min-w-px relative" data-name="Left">
      <Header1 />
    </div>
  );
}

function Label() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">RLM: 346374</p>
      </div>
    </div>
  );
}

function Label1() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">RLM: 346374</p>
      </div>
    </div>
  );
}

function Label2() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">RLM: 346374</p>
      </div>
    </div>
  );
}

function Label3() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">RLM: 346374</p>
      </div>
    </div>
  );
}

function Label4() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">RLM: 346374</p>
      </div>
    </div>
  );
}

function Label5() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">RLM: 346374</p>
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
        <p className="leading-[20px] overflow-hidden">Order Source</p>
      </div>
    </div>
  );
}

function Left1() {
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

function Label6() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">RepairLink</p>
      </div>
    </div>
  );
}

function Label7() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">RepairLink</p>
      </div>
    </div>
  );
}

function Label8() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">RepairLink</p>
      </div>
    </div>
  );
}

function Label9() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">RepairLink</p>
      </div>
    </div>
  );
}

function Label10() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">RepairLink</p>
      </div>
    </div>
  );
}

function Label11() {
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
      <div className="flex flex-col font-['Inter:Bold',sans-serif] font-bold justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#0b0b0b] text-[14px] text-ellipsis whitespace-nowrap">
        <p className="leading-[20px] overflow-hidden">Status</p>
      </div>
    </div>
  );
}

function Left2() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[8px] items-center min-h-px min-w-px relative" data-name="Left">
      <Header3 />
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

function Column() {
  return (
    <div className="bg-white content-stretch flex flex-col items-start relative shrink-0 w-[120px]" data-name="Column">
      <div className="bg-[#eaeaea] content-stretch flex gap-[8px] h-[48px] items-center p-[16px] relative shrink-0 w-[280px]" data-name=".Header">
        <Left2 />
        <div className="-translate-y-1/2 absolute bg-[rgba(24,29,31,0.15)] content-stretch flex h-[14px] items-center left-0 overflow-clip top-1/2 w-[2px]" data-name="Pipe - L">
          <div className="bg-[rgba(24,29,31,0.15)] h-full shrink-0 w-[2px]" data-name="shape" />
        </div>
      </div>
      <div className="h-[48px] relative shrink-0 w-full" data-name=".Accessory cell">
        <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
        <div className="flex flex-row items-center size-full">
          <div className="content-stretch flex items-center px-[16px] py-[8px] relative size-full">
            <div className="bg-[#f3f8f4] content-stretch flex gap-[4px] items-center px-[8px] py-[4px] relative rounded-[999px] shrink-0" data-name="Tag">
              <div aria-hidden="true" className="absolute border border-[#3a6f3b] border-solid inset-0 pointer-events-none rounded-[999px]" />
              <p className="font-['Inter:Bold',sans-serif] font-bold leading-[16px] not-italic relative shrink-0 text-[#202020] text-[12px] whitespace-nowrap">New</p>
            </div>
          </div>
        </div>
      </div>
      <div className="h-[48px] relative shrink-0 w-full" data-name=".Accessory cell">
        <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
        <div className="flex flex-row items-center size-full">
          <div className="content-stretch flex items-center px-[16px] py-[8px] relative size-full">
            <div className="bg-[#fff9e6] content-stretch flex gap-[4px] items-center px-[8px] py-[4px] relative rounded-[999px] shrink-0" data-name="Tag">
              <div aria-hidden="true" className="absolute border border-[#bd5f15] border-solid inset-0 pointer-events-none rounded-[999px]" />
              <p className="font-['Inter:Bold',sans-serif] font-bold leading-[16px] not-italic relative shrink-0 text-[#202020] text-[12px] whitespace-nowrap">Opened</p>
            </div>
          </div>
        </div>
      </div>
      <div className="h-[48px] relative shrink-0 w-full" data-name=".Accessory cell">
        <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
        <div className="flex flex-row items-center size-full">
          <div className="content-stretch flex items-center px-[16px] py-[8px] relative size-full">
            <div className="bg-[#fff9e6] content-stretch flex gap-[4px] items-center px-[8px] py-[4px] relative rounded-[999px] shrink-0" data-name="Tag">
              <div aria-hidden="true" className="absolute border border-[#bd5f15] border-solid inset-0 pointer-events-none rounded-[999px]" />
              <p className="font-['Inter:Bold',sans-serif] font-bold leading-[16px] not-italic relative shrink-0 text-[#202020] text-[12px] whitespace-nowrap">Opened</p>
            </div>
          </div>
        </div>
      </div>
      <div className="h-[48px] relative shrink-0 w-full" data-name=".Accessory cell">
        <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
        <div className="flex flex-row items-center size-full">
          <div className="content-stretch flex items-center px-[16px] py-[8px] relative size-full">
            <div className="bg-[#fff9e6] content-stretch flex gap-[4px] items-center px-[8px] py-[4px] relative rounded-[999px] shrink-0" data-name="Tag">
              <div aria-hidden="true" className="absolute border border-[#bd5f15] border-solid inset-0 pointer-events-none rounded-[999px]" />
              <p className="font-['Inter:Bold',sans-serif] font-bold leading-[16px] not-italic relative shrink-0 text-[#202020] text-[12px] whitespace-nowrap">Opened</p>
            </div>
          </div>
        </div>
      </div>
      <div className="h-[48px] relative shrink-0 w-full" data-name=".Accessory cell">
        <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
        <div className="flex flex-row items-center size-full">
          <div className="content-stretch flex items-center px-[16px] py-[8px] relative size-full">
            <div className="bg-[#fff9e6] content-stretch flex gap-[4px] items-center px-[8px] py-[4px] relative rounded-[999px] shrink-0" data-name="Tag">
              <div aria-hidden="true" className="absolute border border-[#bd5f15] border-solid inset-0 pointer-events-none rounded-[999px]" />
              <p className="font-['Inter:Bold',sans-serif] font-bold leading-[16px] not-italic relative shrink-0 text-[#202020] text-[12px] whitespace-nowrap">Opened</p>
            </div>
          </div>
        </div>
      </div>
      <div className="h-[48px] relative shrink-0 w-full" data-name=".Accessory cell">
        <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
        <div className="flex flex-row items-center size-full">
          <div className="content-stretch flex items-center px-[16px] py-[8px] relative size-full">
            <div className="bg-[#f3f8f4] content-stretch flex gap-[4px] items-center px-[8px] py-[4px] relative rounded-[999px] shrink-0" data-name="Tag">
              <div aria-hidden="true" className="absolute border border-[#3a6f3b] border-solid inset-0 pointer-events-none rounded-[999px]" />
              <p className="font-['Inter:Bold',sans-serif] font-bold leading-[16px] not-italic relative shrink-0 text-[#202020] text-[12px] whitespace-nowrap">New</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function Header4() {
  return (
    <div className="content-stretch flex gap-[4px] items-center relative shrink-0" data-name="Header">
      <div className="flex flex-col font-['Inter:Bold',sans-serif] font-bold justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#0b0b0b] text-[14px] text-ellipsis whitespace-nowrap">
        <p className="leading-[20px] overflow-hidden">Payment</p>
      </div>
    </div>
  );
}

function Left3() {
  return (
    <div className="content-stretch flex gap-[8px] items-center relative shrink-0" data-name="Left">
      <Header4 />
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

function Column1() {
  return (
    <div className="bg-white content-stretch flex flex-col items-start relative self-stretch shrink-0 w-[120px]" data-name="Column">
      <div className="bg-[#eaeaea] h-[48px] relative shrink-0 w-full" data-name=".Header">
        <div className="flex flex-row items-center size-full">
          <div className="content-stretch flex gap-[8px] items-center p-[16px] relative size-full">
            <Left3 />
            <div className="-translate-y-1/2 absolute bg-[rgba(24,29,31,0.15)] content-stretch flex h-[14px] items-center left-0 overflow-clip top-1/2 w-[2px]" data-name="Pipe - L">
              <div className="bg-[rgba(24,29,31,0.15)] h-full shrink-0 w-[2px]" data-name="shape" />
            </div>
          </div>
        </div>
      </div>
      <div className="h-[48px] relative shrink-0 w-full" data-name=".Accessory cell">
        <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
        <div className="flex flex-row items-center size-full">
          <div className="content-stretch flex items-center px-[16px] py-[8px] relative size-full">
            <div className="bg-[#f3f8f4] content-stretch flex gap-[4px] items-center px-[8px] py-[4px] relative rounded-[999px] shrink-0" data-name="Tag">
              <div aria-hidden="true" className="absolute border border-[#3a6f3b] border-solid inset-0 pointer-events-none rounded-[999px]" />
              <p className="font-['Inter:Bold',sans-serif] font-bold leading-[16px] not-italic relative shrink-0 text-[#202020] text-[12px] whitespace-nowrap">Paid</p>
            </div>
          </div>
        </div>
      </div>
      <div className="h-[48px] relative shrink-0 w-full" data-name=".Accessory cell">
        <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
        <div className="flex flex-row items-center size-full">
          <div className="content-stretch flex items-center px-[16px] py-[8px] relative size-full">
            <div className="bg-[#f3f8f4] content-stretch flex gap-[4px] items-center px-[8px] py-[4px] relative rounded-[999px] shrink-0" data-name="Tag">
              <div aria-hidden="true" className="absolute border border-[#3a6f3b] border-solid inset-0 pointer-events-none rounded-[999px]" />
              <p className="font-['Inter:Bold',sans-serif] font-bold leading-[16px] not-italic relative shrink-0 text-[#202020] text-[12px] whitespace-nowrap">Paid</p>
            </div>
          </div>
        </div>
      </div>
      <div className="h-[48px] relative shrink-0 w-full" data-name=".Accessory cell">
        <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
        <div className="flex flex-row items-center size-full">
          <div className="content-stretch flex items-center px-[16px] py-[8px] relative size-full">
            <div className="bg-[#f3f8f4] content-stretch flex gap-[4px] items-center px-[8px] py-[4px] relative rounded-[999px] shrink-0" data-name="Tag">
              <div aria-hidden="true" className="absolute border border-[#3a6f3b] border-solid inset-0 pointer-events-none rounded-[999px]" />
              <p className="font-['Inter:Bold',sans-serif] font-bold leading-[16px] not-italic relative shrink-0 text-[#202020] text-[12px] whitespace-nowrap">Paid</p>
            </div>
          </div>
        </div>
      </div>
      <div className="h-[48px] relative shrink-0 w-full" data-name=".Accessory cell">
        <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
        <div className="flex flex-row items-center size-full">
          <div className="content-stretch flex items-center px-[16px] py-[8px] relative size-full">
            <div className="bg-[#f3f8f4] content-stretch flex gap-[4px] items-center px-[8px] py-[4px] relative rounded-[999px] shrink-0" data-name="Tag">
              <div aria-hidden="true" className="absolute border border-[#3a6f3b] border-solid inset-0 pointer-events-none rounded-[999px]" />
              <p className="font-['Inter:Bold',sans-serif] font-bold leading-[16px] not-italic relative shrink-0 text-[#202020] text-[12px] whitespace-nowrap">Paid</p>
            </div>
          </div>
        </div>
      </div>
      <div className="h-[48px] relative shrink-0 w-full" data-name=".Accessory cell">
        <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
        <div className="flex flex-row items-center size-full">
          <div className="content-stretch flex items-center px-[16px] py-[8px] relative size-full">
            <div className="bg-[#f3f8f4] content-stretch flex gap-[4px] items-center px-[8px] py-[4px] relative rounded-[999px] shrink-0" data-name="Tag">
              <div aria-hidden="true" className="absolute border border-[#3a6f3b] border-solid inset-0 pointer-events-none rounded-[999px]" />
              <p className="font-['Inter:Bold',sans-serif] font-bold leading-[16px] not-italic relative shrink-0 text-[#202020] text-[12px] whitespace-nowrap">Paid</p>
            </div>
          </div>
        </div>
      </div>
      <div className="h-[48px] relative shrink-0 w-full" data-name=".Accessory cell">
        <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
        <div className="flex flex-row items-center size-full">
          <div className="content-stretch flex items-center px-[16px] py-[8px] relative size-full">
            <div className="bg-[#fff9e6] content-stretch flex gap-[4px] items-center px-[8px] py-[4px] relative rounded-[999px] shrink-0" data-name="Tag">
              <div aria-hidden="true" className="absolute border border-[#bd5f15] border-solid inset-0 pointer-events-none rounded-[999px]" />
              <p className="font-['Inter:Bold',sans-serif] font-bold leading-[16px] not-italic relative shrink-0 text-[#202020] text-[12px] whitespace-nowrap">Unpaid</p>
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
      <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / circle-info">
        <div className="absolute inset-[10%]" data-name="Vector">
          <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.8 12.8">
            <path d={svgPaths.p18411100} fill="var(--fill-0, #4D4E53)" id="Vector" />
          </svg>
        </div>
      </div>
      <div className="flex flex-col font-['Inter:Bold',sans-serif] font-bold justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#0b0b0b] text-[14px] text-ellipsis whitespace-nowrap">
        <p className="leading-[20px] overflow-hidden">Submitted By</p>
      </div>
    </div>
  );
}

function Left4() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[8px] items-center min-h-px min-w-px relative" data-name="Left">
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

function Label12() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">Name</p>
      </div>
    </div>
  );
}

function Label13() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">Name</p>
      </div>
    </div>
  );
}

function Label14() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">Name</p>
      </div>
    </div>
  );
}

function Label15() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">Name</p>
      </div>
    </div>
  );
}

function Label16() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">Name</p>
      </div>
    </div>
  );
}

function Label17() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">Name</p>
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
        <p className="leading-[20px] overflow-hidden">{`Vehicle & Cart Name`}</p>
      </div>
    </div>
  );
}

function Left5() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[8px] items-center min-h-px min-w-px relative" data-name="Left">
      <Header6 />
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

function Label18() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">2024 Hyundai Santa FE / Cart 2323</p>
      </div>
    </div>
  );
}

function Label19() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">2024 Hyundai Santa FE / Cart 2323</p>
      </div>
    </div>
  );
}

function Label20() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">2024 Hyundai Santa FE / Cart 2323</p>
      </div>
    </div>
  );
}

function Label21() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">2024 Hyundai Santa FE / Cart 2323</p>
      </div>
    </div>
  );
}

function Label22() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">2024 Hyundai Santa FE / Cart 2323</p>
      </div>
    </div>
  );
}

function Label23() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">2024 Hyundai Santa FE / Cart 2323</p>
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
        <p className="leading-[20px] overflow-hidden">Salesperson</p>
      </div>
    </div>
  );
}

function Left6() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[8px] items-center min-h-px min-w-px relative" data-name="Left">
      <Header7 />
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

function Label24() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">Name</p>
      </div>
    </div>
  );
}

function Label25() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">Name</p>
      </div>
    </div>
  );
}

function Label26() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">Name</p>
      </div>
    </div>
  );
}

function Label27() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">Name</p>
      </div>
    </div>
  );
}

function Label28() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">Name</p>
      </div>
    </div>
  );
}

function Label29() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">Name</p>
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
        <p className="leading-[20px] overflow-hidden">OEM</p>
      </div>
    </div>
  );
}

function Left7() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[8px] items-center min-h-px min-w-px relative" data-name="Left">
      <Header8 />
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

function Label30() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">Lexus</p>
      </div>
    </div>
  );
}

function Label31() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">Toyota</p>
      </div>
    </div>
  );
}

function Label32() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">Mazda</p>
      </div>
    </div>
  );
}

function Label33() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">Toyota</p>
      </div>
    </div>
  );
}

function Label34() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">Mazda</p>
      </div>
    </div>
  );
}

function Label35() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">Lexus</p>
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
        <p className="leading-[20px] overflow-hidden">Photo</p>
      </div>
    </div>
  );
}

function Left8() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[8px] items-center min-h-px min-w-px relative" data-name="Left">
      <Header9 />
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

function Label36() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="overflow-hidden">
          <span className="leading-[24px]">{`Total (4) · `}</span>
          <span className="leading-[24px] text-[#3a6f3b]">New (2)</span>
        </p>
      </div>
    </div>
  );
}

function Label37() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">Total (4)</p>
      </div>
    </div>
  );
}

function Label38() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="overflow-hidden">
          <span className="leading-[24px]">{`Total (4) · `}</span>
          <span className="leading-[24px] text-[#3a6f3b]">New (2)</span>
        </p>
      </div>
    </div>
  );
}

function Label39() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="overflow-hidden">
          <span className="leading-[24px]">{`Total (4) · `}</span>
          <span className="leading-[24px] text-[#3a6f3b]">New (2)</span>
        </p>
      </div>
    </div>
  );
}

function Label40() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">Total (4)</p>
      </div>
    </div>
  );
}

function Label41() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="overflow-hidden">
          <span className="leading-[24px]">{`Total (4) · `}</span>
          <span className="leading-[24px] text-[#3a6f3b]">New (2)</span>
        </p>
      </div>
    </div>
  );
}

function Header10() {
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
        <p className="leading-[20px] overflow-hidden">Notes</p>
      </div>
    </div>
  );
}

function Left9() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[8px] items-center min-h-px min-w-px relative" data-name="Left">
      <Header10 />
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

function Label42() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="overflow-hidden">
          <span className="leading-[24px]">{`Total (4) · `}</span>
          <span className="leading-[24px] text-[#3a6f3b]">New (2)</span>
        </p>
      </div>
    </div>
  );
}

function Label43() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="overflow-hidden">
          <span className="leading-[24px]">{`Total (4) · `}</span>
          <span className="leading-[24px] text-[#3a6f3b]">New (2)</span>
        </p>
      </div>
    </div>
  );
}

function Label44() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">Total (4)</p>
      </div>
    </div>
  );
}

function Label45() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">Total (4)</p>
      </div>
    </div>
  );
}

function Label46() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">Total (4)</p>
      </div>
    </div>
  );
}

function Label47() {
  return (
    <div className="content-stretch flex flex-[1_0_0] flex-col items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic overflow-hidden relative shrink-0 text-[#202020] text-[16px] text-ellipsis w-full whitespace-nowrap">
        <p className="leading-[24px] overflow-hidden">Total (4)</p>
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
        <p className="leading-[20px] overflow-hidden">Notes</p>
      </div>
    </div>
  );
}

function Left10() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[8px] items-center min-h-px min-w-px relative" data-name="Left">
      <Header11 />
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

function Label48() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">Memo</p>
      </div>
    </div>
  );
}

function Label49() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">Memo</p>
      </div>
    </div>
  );
}

function Label50() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">Memo</p>
      </div>
    </div>
  );
}

function Label51() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">Memo</p>
      </div>
    </div>
  );
}

function Label52() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">Memo</p>
      </div>
    </div>
  );
}

function Label53() {
  return (
    <div className="content-stretch flex flex-[1_0_0] gap-[4px] items-center justify-center min-h-px min-w-px relative" data-name="Label">
      <div className="flex flex-[1_0_0] flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-h-px min-w-px not-italic relative text-[#202020] text-[16px]">
        <p className="leading-[24px]">Memo</p>
      </div>
    </div>
  );
}

function Header12() {
  return <div className="bg-[#eaeaea] content-stretch flex gap-[8px] h-[48px] items-center p-[16px] shrink-0 w-[56px]" data-name=".Header" />;
}

function Column2() {
  return (
    <div className="bg-white content-stretch flex flex-col items-start relative shrink-0" data-name="Column">
      <Header12 />
      <div className="content-stretch flex items-center px-[16px] py-[12px] relative shrink-0" data-name=".Accessory cell">
        <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
        <div className="overflow-clip relative shrink-0 size-[24px]" data-name="Icon / ellipsis-vertical">
          <div className="absolute inset-[15%_41.25%]" data-name="Vector">
            <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 4.2 16.8">
              <path d={svgPaths.p13378a00} fill="var(--fill-0, #4D4E53)" id="Vector" />
            </svg>
          </div>
        </div>
      </div>
      <div className="content-stretch flex items-center px-[16px] py-[12px] relative shrink-0" data-name=".Accessory cell">
        <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
        <div className="overflow-clip relative shrink-0 size-[24px]" data-name="Icon / ellipsis-vertical">
          <div className="absolute inset-[15%_41.25%]" data-name="Vector">
            <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 4.2 16.8">
              <path d={svgPaths.p13378a00} fill="var(--fill-0, #4D4E53)" id="Vector" />
            </svg>
          </div>
        </div>
      </div>
      <div className="content-stretch flex items-center px-[16px] py-[12px] relative shrink-0" data-name=".Accessory cell">
        <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
        <div className="overflow-clip relative shrink-0 size-[24px]" data-name="Icon / ellipsis-vertical">
          <div className="absolute inset-[15%_41.25%]" data-name="Vector">
            <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 4.2 16.8">
              <path d={svgPaths.p13378a00} fill="var(--fill-0, #4D4E53)" id="Vector" />
            </svg>
          </div>
        </div>
      </div>
      <div className="content-stretch flex items-center px-[16px] py-[12px] relative shrink-0" data-name=".Accessory cell">
        <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
        <div className="overflow-clip relative shrink-0 size-[24px]" data-name="Icon / ellipsis-vertical">
          <div className="absolute inset-[15%_41.25%]" data-name="Vector">
            <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 4.2 16.8">
              <path d={svgPaths.p13378a00} fill="var(--fill-0, #4D4E53)" id="Vector" />
            </svg>
          </div>
        </div>
      </div>
      <div className="content-stretch flex items-center px-[16px] py-[12px] relative shrink-0" data-name=".Accessory cell">
        <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
        <div className="overflow-clip relative shrink-0 size-[24px]" data-name="Icon / ellipsis-vertical">
          <div className="absolute inset-[15%_41.25%]" data-name="Vector">
            <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 4.2 16.8">
              <path d={svgPaths.p13378a00} fill="var(--fill-0, #4D4E53)" id="Vector" />
            </svg>
          </div>
        </div>
      </div>
      <div className="content-stretch flex items-center px-[16px] py-[12px] relative shrink-0" data-name=".Accessory cell">
        <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
        <div className="overflow-clip relative shrink-0 size-[24px]" data-name="Icon / ellipsis-vertical">
          <div className="absolute inset-[15%_41.25%]" data-name="Vector">
            <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 4.2 16.8">
              <path d={svgPaths.p13378a00} fill="var(--fill-0, #4D4E53)" id="Vector" />
            </svg>
          </div>
        </div>
      </div>
    </div>
  );
}

function DataTable() {
  return (
    <div className="bg-white relative rounded-[8px] shrink-0 w-[2143px]" data-name="Data table">
      <div className="content-stretch flex items-start overflow-clip relative rounded-[inherit] w-full">
        <div className="bg-white content-stretch flex flex-col items-start relative self-stretch shrink-0 w-[200px]" data-name="Column">
          <div className="bg-[#eaeaea] h-[48px] relative shrink-0 w-full" data-name=".Header">
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center p-[16px] relative size-full">
                <Left />
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
        <div className="bg-white content-stretch flex flex-col items-start relative self-stretch shrink-0 w-[180px]" data-name="Column">
          <div className="bg-[#eaeaea] h-[48px] relative shrink-0 w-full" data-name=".Header">
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center p-[16px] relative size-full">
                <Left1 />
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
                <Label6 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <Label7 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <Label8 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <Label9 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <Label10 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <Label11 />
              </div>
            </div>
          </div>
        </div>
        <Column />
        <Column1 />
        <div className="bg-white content-stretch flex flex-[1_0_0] flex-col items-start min-h-px min-w-px relative self-stretch" data-name="Column">
          <div className="bg-[#eaeaea] h-[48px] relative shrink-0 w-full" data-name=".Header">
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center p-[16px] relative size-full">
                <Left4 />
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
                <Label12 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <Label13 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <Label14 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <Label15 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <Label16 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <Label17 />
              </div>
            </div>
          </div>
        </div>
        <div className="bg-white content-stretch flex flex-col items-start relative self-stretch shrink-0 w-[292px]" data-name="Column">
          <div className="bg-[#eaeaea] h-[48px] relative shrink-0 w-full" data-name=".Header">
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center p-[16px] relative size-full">
                <Left5 />
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
        <div className="bg-white content-stretch flex flex-col items-start relative self-stretch shrink-0 w-[220px]" data-name="Column">
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
        <div className="bg-white content-stretch flex flex-col items-start relative self-stretch shrink-0 w-[220px]" data-name="Column">
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
                <Label30 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <Label31 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <Label32 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <Label33 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <Label34 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <Label35 />
              </div>
            </div>
          </div>
        </div>
        <div className="bg-white content-stretch flex flex-col items-start relative self-stretch shrink-0 w-[200px]" data-name="Column">
          <div className="bg-[#eaeaea] h-[48px] relative shrink-0 w-full" data-name=".Header">
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center p-[16px] relative size-full">
                <Left8 />
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
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / link-simple">
                  <div className="absolute inset-[20%_5%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 14.4 9.6">
                      <path d={svgPaths.p3a2d1100} fill="var(--fill-0, #656565)" id="Vector" />
                    </svg>
                  </div>
                </div>
                <Label36 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / link-simple">
                  <div className="absolute inset-[20%_5%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 14.4 9.6">
                      <path d={svgPaths.p3a2d1100} fill="var(--fill-0, #656565)" id="Vector" />
                    </svg>
                  </div>
                </div>
                <Label37 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / link-simple">
                  <div className="absolute inset-[20%_5%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 14.4 9.6">
                      <path d={svgPaths.p3a2d1100} fill="var(--fill-0, #656565)" id="Vector" />
                    </svg>
                  </div>
                </div>
                <Label38 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / link-simple">
                  <div className="absolute inset-[20%_5%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 14.4 9.6">
                      <path d={svgPaths.p3a2d1100} fill="var(--fill-0, #656565)" id="Vector" />
                    </svg>
                  </div>
                </div>
                <Label39 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / link-simple">
                  <div className="absolute inset-[20%_5%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 14.4 9.6">
                      <path d={svgPaths.p3a2d1100} fill="var(--fill-0, #656565)" id="Vector" />
                    </svg>
                  </div>
                </div>
                <Label40 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / link-simple">
                  <div className="absolute inset-[20%_5%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 14.4 9.6">
                      <path d={svgPaths.p3a2d1100} fill="var(--fill-0, #656565)" id="Vector" />
                    </svg>
                  </div>
                </div>
                <Label41 />
              </div>
            </div>
          </div>
        </div>
        <div className="bg-white content-stretch flex flex-col items-start relative self-stretch shrink-0 w-[200px]" data-name="Column">
          <div className="bg-[#eaeaea] h-[48px] relative shrink-0 w-full" data-name=".Header">
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[8px] items-center p-[16px] relative size-full">
                <Left9 />
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
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / link-simple">
                  <div className="absolute inset-[20%_5%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 14.4 9.6">
                      <path d={svgPaths.p3a2d1100} fill="var(--fill-0, #656565)" id="Vector" />
                    </svg>
                  </div>
                </div>
                <Label42 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / link-simple">
                  <div className="absolute inset-[20%_5%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 14.4 9.6">
                      <path d={svgPaths.p3a2d1100} fill="var(--fill-0, #656565)" id="Vector" />
                    </svg>
                  </div>
                </div>
                <Label43 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / link-simple">
                  <div className="absolute inset-[20%_5%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 14.4 9.6">
                      <path d={svgPaths.p3a2d1100} fill="var(--fill-0, #656565)" id="Vector" />
                    </svg>
                  </div>
                </div>
                <Label44 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / link-simple">
                  <div className="absolute inset-[20%_5%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 14.4 9.6">
                      <path d={svgPaths.p3a2d1100} fill="var(--fill-0, #656565)" id="Vector" />
                    </svg>
                  </div>
                </div>
                <Label45 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / link-simple">
                  <div className="absolute inset-[20%_5%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 14.4 9.6">
                      <path d={svgPaths.p3a2d1100} fill="var(--fill-0, #656565)" id="Vector" />
                    </svg>
                  </div>
                </div>
                <Label46 />
              </div>
            </div>
          </div>
          <div className="relative shrink-0 w-full" data-name=".Static cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <div className="flex flex-row items-center size-full">
              <div className="content-stretch flex gap-[4px] items-center px-[16px] py-[12px] relative w-full">
                <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / link-simple">
                  <div className="absolute inset-[20%_5%]" data-name="Vector">
                    <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 14.4 9.6">
                      <path d={svgPaths.p3a2d1100} fill="var(--fill-0, #656565)" id="Vector" />
                    </svg>
                  </div>
                </div>
                <Label47 />
              </div>
            </div>
          </div>
        </div>
        <div className="bg-white content-stretch flex flex-[1_0_0] flex-col items-start min-h-px min-w-px relative self-stretch" data-name="Column">
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
          <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative shrink-0 w-[280px]" data-name=".Interactive cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <Label48 />
            <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / pen-to-square">
              <div className="absolute inset-[10.82%_10.82%_10%_10%]" data-name="Vector">
                <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.6681 12.6681">
                  <path d={svgPaths.p3c006800} fill="var(--fill-0, #4D4E53)" id="Vector" />
                </svg>
              </div>
            </div>
          </div>
          <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative shrink-0 w-[280px]" data-name=".Interactive cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <Label49 />
            <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / pen-to-square">
              <div className="absolute inset-[10.82%_10.82%_10%_10%]" data-name="Vector">
                <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.6681 12.6681">
                  <path d={svgPaths.p3c006800} fill="var(--fill-0, #4D4E53)" id="Vector" />
                </svg>
              </div>
            </div>
          </div>
          <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative shrink-0 w-[280px]" data-name=".Interactive cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <Label50 />
            <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / pen-to-square">
              <div className="absolute inset-[10.82%_10.82%_10%_10%]" data-name="Vector">
                <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.6681 12.6681">
                  <path d={svgPaths.p3c006800} fill="var(--fill-0, #4D4E53)" id="Vector" />
                </svg>
              </div>
            </div>
          </div>
          <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative shrink-0 w-[280px]" data-name=".Interactive cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <Label51 />
            <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / pen-to-square">
              <div className="absolute inset-[10.82%_10.82%_10%_10%]" data-name="Vector">
                <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.6681 12.6681">
                  <path d={svgPaths.p3c006800} fill="var(--fill-0, #4D4E53)" id="Vector" />
                </svg>
              </div>
            </div>
          </div>
          <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative shrink-0 w-[280px]" data-name=".Interactive cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
            <Label52 />
            <div className="overflow-clip relative shrink-0 size-[16px]" data-name="Icon / pen-to-square">
              <div className="absolute inset-[10.82%_10.82%_10%_10%]" data-name="Vector">
                <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 12.6681 12.6681">
                  <path d={svgPaths.p3c006800} fill="var(--fill-0, #4D4E53)" id="Vector" />
                </svg>
              </div>
            </div>
          </div>
          <div className="content-stretch flex gap-[8px] items-center px-[16px] py-[12px] relative shrink-0 w-[280px]" data-name=".Interactive cell">
            <div aria-hidden="true" className="absolute border-[#dcdcdc] border-b border-solid inset-0 pointer-events-none" />
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
        <Column2 />
      </div>
      <div aria-hidden="true" className="absolute border border-[#dcdcdc] border-solid inset-0 pointer-events-none rounded-[8px]" />
    </div>
  );
}

function Container() {
  return (
    <div className="absolute content-stretch flex flex-col gap-[16px] items-start left-[204px] top-[181px] w-[2143px]" data-name="Container">
      <Frame3 />
      <div className="bg-white content-stretch flex h-[40px] items-start p-[4px] relative rounded-[4px] shrink-0 w-[385px]" data-name="Segmented control (In progress)">
        <div aria-hidden="true" className="absolute border border-[#c4c4c4] border-solid inset-0 pointer-events-none rounded-[4px]" />
        <div className="flex-[1_0_0] min-h-px min-w-px relative rounded-[4px]" data-name=".Segements">
          <div className="flex flex-row items-center justify-center size-full">
            <div className="content-stretch flex gap-[4px] items-center justify-center px-[8px] py-[4px] relative w-full">
              <p className="font-['Inter:Bold',sans-serif] font-bold leading-[24px] not-italic relative shrink-0 text-[#003c71] text-[16px] whitespace-nowrap">All (15)</p>
            </div>
          </div>
        </div>
        <div className="bg-[#003c71] flex-[1_0_0] min-h-px min-w-px relative rounded-[4px]" data-name=".Segements">
          <div className="flex flex-row items-center justify-center size-full">
            <div className="content-stretch flex gap-[4px] items-center justify-center px-[8px] py-[4px] relative w-full">
              <p className="font-['Inter:Bold',sans-serif] font-bold leading-[24px] not-italic relative shrink-0 text-[16px] text-white whitespace-nowrap">Open (8)</p>
            </div>
          </div>
        </div>
        <div className="flex-[1_0_0] min-h-px min-w-px relative rounded-[4px]" data-name=".Segements">
          <div className="flex flex-row items-center justify-center size-full">
            <div className="content-stretch flex gap-[4px] items-center justify-center px-[8px] py-[4px] relative w-full">
              <p className="font-['Inter:Bold',sans-serif] font-bold leading-[24px] not-italic relative shrink-0 text-[#003c71] text-[16px] whitespace-nowrap">Closed (7)</p>
            </div>
          </div>
        </div>
      </div>
      <Frame />
      <DataTable />
    </div>
  );
}

export default function TransactionsAll() {
  return (
    <div className="bg-white relative size-full" data-name="Transactions (All)">
      <Header />
      <Sidebar />
      <Container />
    </div>
  );
}