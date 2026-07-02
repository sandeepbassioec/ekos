import svgPaths from "./svg-bksr1sztf1";
import imgPhoto from "figma:asset/7f12ea1300756f144a0fb5daaf68dbfc01103a46.png";

function Frame6() {
  return (
    <div className="content-stretch flex flex-col items-start relative shrink-0">
      <p className="font-['Inter:Bold',sans-serif] font-bold leading-[24px] min-w-full not-italic relative shrink-0 text-[#4d4e53] text-[16px] w-[min-content]">Vehicle details</p>
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

function AddPhotoAlternate24Dp5F6368Fill1Wght400Grad0Opsz() {
  return <div className="shrink-0 size-[24px]" data-name="add_photo_alternate_24dp_5F6368_FILL1_wght400_GRAD0_opsz24 1" />;
}

function Frame() {
  return (
    <div className="content-stretch flex items-center justify-between relative shrink-0 w-full">
      <div className="flex flex-col font-['Inter:Bold',sans-serif] font-bold justify-end leading-[0] not-italic relative shrink-0 text-[#202020] text-[16px] whitespace-nowrap">
        <p className="leading-[24px]">Photos</p>
      </div>
      <AddPhotoAlternate24Dp5F6368Fill1Wght400Grad0Opsz />
    </div>
  );
}

function Frame5() {
  return (
    <div className="bg-white h-full relative rounded-[8px] shrink-0 w-[179px]">
      <div aria-hidden="true" className="absolute border border-[#eaeaea] border-solid inset-0 pointer-events-none rounded-[8px]" />
      <div className="flex flex-col items-center justify-center size-full">
        <div className="content-stretch flex flex-col gap-[16px] items-center justify-center p-[16px] relative size-full">
          <div className="relative shrink-0 size-[18px]" data-name="Vector">
            <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 18 18">
              <path d={svgPaths.p33c0fe00} fill="var(--fill-0, #5F6368)" id="Vector" />
            </svg>
          </div>
          <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] min-w-full not-italic relative shrink-0 text-[#161616] text-[14px] text-center w-[min-content]">
            <p className="leading-[20px]">Upload new photos</p>
          </div>
          <div className="bg-[#003c71] content-stretch flex gap-[4px] h-[32px] items-center justify-center px-[16px] py-[8px] relative rounded-[8px] shrink-0" data-name="OButton">
            <div className="content-stretch flex items-center justify-center overflow-clip relative shrink-0 size-[12px]" data-name="OIcon">
              <div className="aspect-[40/40] flex-[1_0_0] min-h-px min-w-px relative" data-name="upload">
                <div className="-translate-y-1/2 absolute aspect-[38/40] left-[2.5%] right-[2.5%] top-1/2" data-name="Vector">
                  <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 11.4 12">
                    <path d={svgPaths.p3990d000} fill="var(--fill-0, white)" id="Vector" />
                  </svg>
                </div>
              </div>
            </div>
            <p className="font-['Inter:Bold',sans-serif] font-bold leading-[20px] not-italic relative shrink-0 text-[14px] text-white whitespace-nowrap">Upload</p>
          </div>
        </div>
      </div>
    </div>
  );
}

function Photo() {
  return (
    <div className="h-[131px] relative rounded-[8px] shrink-0 w-full" data-name="Photo">
      <img alt="" className="absolute inset-0 max-w-none object-cover pointer-events-none rounded-[8px] size-full" src={imgPhoto} />
    </div>
  );
}

function PhotoDetails() {
  return (
    <div className="content-stretch flex flex-col font-['Inter:Regular',sans-serif] font-normal gap-[4px] items-start leading-[0] not-italic relative shrink-0 text-[#161616] text-[0px] text-ellipsis w-full whitespace-nowrap" data-name="Photo details">
      <div className="flex flex-col justify-center overflow-hidden relative shrink-0 w-full">
        <p className="overflow-hidden text-[14px]">
          <span className="leading-[20px] text-[#656565]">Added by:</span>
          <span className="leading-[20px] text-[#595959]">{` `}</span>
          <span className="leading-[20px] text-[#202020]">OEConnection test acc</span>
          <span className="leading-[20px]">ount</span>
        </p>
      </div>
      <div className="flex flex-col justify-center overflow-hidden relative shrink-0 w-full">
        <p className="overflow-hidden text-[14px]">
          <span className="leading-[20px] text-[#656565]">Last Modified:</span>
          <span className="leading-[20px] text-[#595959]">{` `}</span>
          <span className="leading-[20px] text-[#202020]">7/23/2025 10:40:1</span>
          <span className="leading-[20px]">2 AM</span>
        </p>
      </div>
    </div>
  );
}

function PhotoCard() {
  return (
    <div className="content-stretch flex flex-col gap-[12px] items-start p-[8px] relative rounded-[8px] shrink-0 w-[241px]" data-name="Photo card">
      <div aria-hidden="true" className="absolute border border-[#eaeaea] border-solid inset-0 pointer-events-none rounded-[8px]" />
      <Photo />
      <PhotoDetails />
    </div>
  );
}

function Photo1() {
  return (
    <div className="h-[131px] relative rounded-[8px] shrink-0 w-full" data-name="Photo">
      <img alt="" className="absolute inset-0 max-w-none object-cover pointer-events-none rounded-[8px] size-full" src={imgPhoto} />
    </div>
  );
}

function PhotoDetails1() {
  return (
    <div className="content-stretch flex flex-col font-['Inter:Regular',sans-serif] font-normal gap-[4px] items-start leading-[0] not-italic relative shrink-0 text-[#161616] text-[0px] text-ellipsis w-full whitespace-nowrap" data-name="Photo details">
      <div className="flex flex-col justify-center overflow-hidden relative shrink-0 w-full">
        <p className="overflow-hidden text-[14px]">
          <span className="leading-[20px] text-[#656565]">Added by:</span>
          <span className="leading-[20px] text-[#595959]">{` `}</span>
          <span className="leading-[20px] text-[#202020]">OEConnection test acc</span>
          <span className="leading-[20px]">ount</span>
        </p>
      </div>
      <div className="flex flex-col justify-center overflow-hidden relative shrink-0 w-full">
        <p className="overflow-hidden text-[14px]">
          <span className="leading-[20px] text-[#656565]">Last Modified:</span>
          <span className="leading-[20px] text-[#595959]">{` `}</span>
          <span className="leading-[20px] text-[#202020]">7/23/2025 10:40:1</span>
          <span className="leading-[20px]">2 AM</span>
        </p>
      </div>
    </div>
  );
}

function PhotoCard1() {
  return (
    <div className="content-stretch flex flex-col gap-[12px] items-start p-[8px] relative rounded-[8px] shrink-0 w-[241px]" data-name="Photo card">
      <div aria-hidden="true" className="absolute border border-[#eaeaea] border-solid inset-0 pointer-events-none rounded-[8px]" />
      <Photo1 />
      <PhotoDetails1 />
    </div>
  );
}

function PhotosRow() {
  return (
    <div className="content-stretch flex gap-[12px] items-center overflow-clip relative rounded-[8px] shrink-0 w-full" data-name="Photos row">
      <div className="flex flex-row items-center self-stretch">
        <Frame5 />
      </div>
      <PhotoCard />
      <PhotoCard1 />
    </div>
  );
}

function PhotosContainer() {
  return (
    <div className="bg-white relative rounded-[8px] shrink-0 w-[752px]" data-name="Photos container">
      <div className="content-stretch flex flex-col gap-[16px] items-start overflow-clip p-[16px] relative rounded-[inherit] w-full">
        <Frame />
        <PhotosRow />
      </div>
      <div aria-hidden="true" className="absolute border border-[#eaeaea] border-solid inset-0 pointer-events-none rounded-[8px]" />
    </div>
  );
}

function Frame1() {
  return (
    <div className="content-stretch flex items-center relative shrink-0 w-full">
      <div className="flex flex-col font-['Inter:Bold',sans-serif] font-bold justify-end leading-[0] not-italic relative shrink-0 text-[#161616] text-[16px] whitespace-nowrap">
        <p className="leading-[24px]">Notes</p>
      </div>
    </div>
  );
}

function Body1() {
  return (
    <div className="bg-white content-stretch flex gap-[2px] items-center justify-center not-italic px-[4px] relative rounded-[4px] shrink-0 text-[12px] whitespace-nowrap" data-name="Body">
      <p className="font-['Inter:Regular',sans-serif] font-normal leading-[16px] relative shrink-0 text-[#202020]">Notes</p>
      <div className="flex flex-col font-['Inter:Bold',sans-serif] font-bold justify-center leading-[0] relative shrink-0 text-[#c23f2e] uppercase">
        <p className="leading-[16px]">*</p>
      </div>
    </div>
  );
}

function Label() {
  return (
    <div className="absolute content-stretch flex flex-col items-start left-0 px-[8px] top-[-10px] w-[304px] z-[2]" data-name="Label">
      <Body1 />
    </div>
  );
}

function CornerResize() {
  return (
    <div className="absolute bottom-[4px] right-[4px] size-[10px]" data-name="Corner resize">
      <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 10 10">
        <g id="Corner resize">
          <path d="M2 8.5L8.5 2" id="Vector 4" stroke="var(--stroke-0, #4D4E53)" strokeLinecap="round" />
          <path d="M6 9L9 6" id="Vector 5" stroke="var(--stroke-0, #4D4E53)" strokeLinecap="round" />
        </g>
      </svg>
    </div>
  );
}

function InputContainer() {
  return (
    <div className="bg-white min-h-[56px] relative rounded-[8px] shrink-0 w-full z-[1]" data-name="Input Container">
      <div aria-hidden="true" className="absolute border border-[#bfbfbf] border-solid inset-0 pointer-events-none rounded-[8px]" />
      <div className="content-stretch flex gap-[8px] items-start min-h-[inherit] px-[16px] py-[8px] relative w-full">
        <p className="font-['Inter:Regular',sans-serif] font-normal leading-[20px] not-italic relative shrink-0 text-[#656565] text-[14px] whitespace-nowrap">Add your note here</p>
        <CornerResize />
      </div>
    </div>
  );
}

function TextField() {
  return (
    <div className="content-stretch flex flex-col isolate items-start relative shrink-0 w-full" data-name="Text Field">
      <Label />
      <InputContainer />
    </div>
  );
}

function Frame2() {
  return (
    <div className="content-stretch flex gap-[4px] items-center relative shrink-0 w-full">
      <div className="relative shrink-0 size-[8px]">
        <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 8 8">
          <circle cx="4" cy="4" fill="var(--fill-0, #3A6F3B)" id="Ellipse 7" r="4" />
        </svg>
      </div>
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic relative shrink-0 text-[#161616] text-[14px] whitespace-nowrap">
        <p className="leading-[20px]">Test note 1</p>
      </div>
    </div>
  );
}

function NoteCard() {
  return (
    <div className="relative shrink-0 w-full" data-name="Note card">
      <div className="content-stretch flex flex-col items-start px-[16px] py-[8px] relative w-full">
        <Frame2 />
        <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic relative shrink-0 text-[#656565] text-[14px] whitespace-nowrap">
          <p className="leading-[20px]">07/23/2025 10:14:30 AM</p>
        </div>
      </div>
    </div>
  );
}

function Frame3() {
  return (
    <div className="content-stretch flex gap-[4px] items-center relative shrink-0 w-full">
      <div className="relative shrink-0 size-[8px]">
        <svg className="absolute block size-full" fill="none" preserveAspectRatio="none" viewBox="0 0 8 8">
          <circle cx="4" cy="4" fill="var(--fill-0, #3A6F3B)" id="Ellipse 7" r="4" />
        </svg>
      </div>
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic relative shrink-0 text-[#161616] text-[14px] whitespace-nowrap">
        <p className="leading-[20px]">Test note 1</p>
      </div>
    </div>
  );
}

function NoteCard1() {
  return (
    <div className="relative shrink-0 w-full" data-name="Note card">
      <div className="content-stretch flex flex-col items-start px-[16px] py-[8px] relative w-full">
        <Frame3 />
        <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic relative shrink-0 text-[#656565] text-[14px] whitespace-nowrap">
          <p className="leading-[20px]">07/23/2025 10:14:30 AM</p>
        </div>
      </div>
    </div>
  );
}

function Frame4() {
  return (
    <div className="content-stretch flex items-center relative shrink-0 w-full">
      <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic relative shrink-0 text-[#161616] text-[14px] whitespace-nowrap">
        <p className="leading-[20px]">Test note 1</p>
      </div>
    </div>
  );
}

function NoteCard2() {
  return (
    <div className="relative shrink-0 w-full" data-name="Note card">
      <div className="content-stretch flex flex-col items-start px-[16px] py-[8px] relative w-full">
        <Frame4 />
        <div className="flex flex-col font-['Inter:Regular',sans-serif] font-normal justify-center leading-[0] not-italic relative shrink-0 text-[#656565] text-[14px] whitespace-nowrap">
          <p className="leading-[20px]">07/23/2025 10:14:30 AM</p>
        </div>
      </div>
    </div>
  );
}

function NotesList() {
  return (
    <div className="content-stretch flex flex-col gap-[4px] items-start relative shrink-0 w-[196px]" data-name="Notes list">
      <NoteCard />
      <NoteCard1 />
      <NoteCard2 />
    </div>
  );
}

function NotesContainer() {
  return (
    <div className="relative rounded-[8px] shrink-0 w-full" data-name="Notes container">
      <div aria-hidden="true" className="absolute border border-[#eaeaea] border-solid inset-0 pointer-events-none rounded-[8px]" />
      <div className="content-stretch flex flex-col gap-[16px] items-start p-[16px] relative w-full">
        <Frame1 />
        <div className="content-stretch flex flex-col gap-[4px] items-start relative shrink-0 w-full" data-name="Text area field">
          <TextField />
        </div>
        <div className="bg-[#003c71] content-stretch flex h-[32px] items-center justify-center px-[16px] py-[8px] relative rounded-[8px] shrink-0" data-name="OButton">
          <p className="font-['Inter:Bold',sans-serif] font-bold leading-[20px] not-italic relative shrink-0 text-[14px] text-white whitespace-nowrap">Save note</p>
        </div>
        <NotesList />
      </div>
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
            <div className="flex-[1_0_0] min-h-px min-w-px relative rounded-[4px]" data-name=".Segements">
              <div className="flex flex-row items-center justify-center size-full">
                <div className="content-stretch flex gap-[4px] items-center justify-center px-[8px] py-[4px] relative w-full">
                  <p className="font-['Inter:Bold',sans-serif] font-bold leading-[24px] not-italic relative shrink-0 text-[#003c71] text-[16px] whitespace-nowrap">Order Info</p>
                </div>
              </div>
            </div>
            <div className="bg-[#003c71] flex-[1_0_0] min-h-px min-w-px relative rounded-[4px]" data-name=".Segements">
              <div className="flex flex-row items-center justify-center size-full">
                <div className="content-stretch flex gap-[4px] items-center justify-center px-[8px] py-[4px] relative w-full">
                  <p className="font-['Inter:Bold',sans-serif] font-bold leading-[24px] not-italic relative shrink-0 text-[16px] text-white whitespace-nowrap">{`Photos & Notes`}</p>
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
          <PhotosContainer />
          <NotesContainer />
        </div>
      </div>
    </div>
  );
}

export default function TableLinePhotosNotes() {
  return (
    <div className="content-stretch flex flex-col items-start relative size-full" data-name="Table Line (Photos & Notes)">
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