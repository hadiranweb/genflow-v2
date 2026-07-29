import { useState } from "react";
import { useNavigate } from "@remix-run/react";
import type { MetaFunction } from "@remix-run/node";
import { GenButton, GenCard, GenInput, GenBadge } from "@genflow/ui";

export const meta: MetaFunction = () => {
  return [
    { title: "GenFlow — تولید موقعیت‌های شغلی هوشمند" },
    { name: "description", content: "پلتفرم هوشمند تولید و تطبیق موقعیت‌های شغلی با معماری هیبریدی" },
    { name: "viewport", content: "width=device-width, initial-scale=1" },
  ];
};

type Step = "business" | "details" | "review";

export default function Index() {
  const navigate = useNavigate();
  const [step, setStep] = useState<Step>("business");
  const [form, setForm] = useState({
    businessName: "",
    industry: "",
    department: "",
    experience: "",
    location: "",
    description: "",
  });
  const [generating, setGenerating] = useState(false);

  const updateField = (field: string, value: string) =>
    setForm((prev) => ({ ...prev, [field]: value }));

  const steps: { id: Step; label: string; labelFa: string }[] = [
    { id: "business", label: "Business Info", labelFa: "اطلاعات کسب‌وکار" },
    { id: "details", label: "Position Details", labelFa: "جزئیات موقعیت" },
    { id: "review", label: "Review", labelFa: "بررسی نهایی" },
  ];

  const currentIdx = steps.findIndex((s) => s.id === step);

  const handleGenerate = async () => {
    setGenerating(true);
    try {
      // Will call the actual API when backend is running
      await new Promise((r) => setTimeout(r, 1500));
      navigate("/positions");
    } catch (err) {
      console.error("Generation failed", err);
    } finally {
      setGenerating(false);
    }
  };

  return (
    <div className="min-h-screen bg-gradient-to-br from-gray-50 to-teal-50" dir="rtl">
      {/* Header */}
      <header className="bg-navy-800 text-white shadow-lg">
        <div className="max-w-6xl mx-auto px-4 py-4 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 bg-teal-500 rounded-lg flex items-center justify-center font-bold text-lg">
              G
            </div>
            <div>
              <h1 className="text-xl font-bold">GenFlow</h1>
              <p className="text-xs text-teal-200">پلتفرم هوشمند منابع انسانی</p>
            </div>
          </div>
          <GenBadge variant="success">فعال</GenBadge>
        </div>
      </header>

      <main className="max-w-4xl mx-auto px-4 py-8">
        {/* Stepper */}
        <nav className="flex items-center justify-center gap-2 mb-10" aria-label="مراحل">
          {steps.map((s, i) => (
            <div key={s.id} className="flex items-center gap-2">
              <button
                onClick={() => i < currentIdx && setStep(s.id)}
                className={`w-10 h-10 rounded-full flex items-center justify-center text-sm font-bold transition-all ${
                  i <= currentIdx
                    ? "bg-teal-500 text-white shadow-md"
                    : "bg-gray-200 text-gray-500"
                }`}
                disabled={i > currentIdx}
              >
                {i + 1}
              </button>
              <span
                className={`text-sm hidden sm:inline ${
                  i <= currentIdx ? "text-teal-700 font-medium" : "text-gray-400"
                }`}
              >
                {s.labelFa}
              </span>
              {i < steps.length - 1 && (
                <div
                  className={`w-12 h-0.5 ${
                    i < currentIdx ? "bg-teal-500" : "bg-gray-200"
                  }`}
                />
              )}
            </div>
          ))}
        </nav>

        {/* Step Content */}
        <GenCard variant="elevated" className="p-8">
          {step === "business" && (
            <div className="space-y-6">
              <h2 className="text-2xl font-bold text-navy-800">اطلاعات کسب‌وکار</h2>
              <p className="text-gray-600">لطفاً اطلاعات پایه‌ای کسب‌وکار خود را وارد کنید.</p>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <GenInput
                  label="نام شرکت / کسب‌وکار"
                  placeholder="مثال: شرکت فناوری نوین"
                  value={form.businessName}
                  onChange={(e) => updateField("businessName", e.target.value)}
                />
                <GenInput
                  label="صنعت / حوزه فعالیت"
                  placeholder="مثال: فناوری اطلاعات"
                  value={form.industry}
                  onChange={(e) => updateField("industry", e.target.value)}
                />
                <GenInput
                  label="دپارتمان"
                  placeholder="مثال: مهندسی"
                  value={form.department}
                  onChange={(e) => updateField("department", e.target.value)}
                />
                <GenInput
                  label="مکان"
                  placeholder="مثال: تهران"
                  value={form.location}
                  onChange={(e) => updateField("location", e.target.value)}
                />
              </div>
            </div>
          )}

          {step === "details" && (
            <div className="space-y-6">
              <h2 className="text-2xl font-bold text-navy-800">جزئیات موقعیت شغلی</h2>
              <p className="text-gray-600">نیازمندی‌های دقیق موقعیت مورد نظر را مشخص کنید.</p>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <GenInput
                  label="سابقه کاری مورد نیاز (سال)"
                  placeholder="مثال: ۳"
                  value={form.experience}
                  onChange={(e) => updateField("experience", e.target.value)}
                />
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  شرح موقعیت شغلی
                </label>
                <textarea
                  className="w-full border border-gray-300 rounded-lg px-4 py-3 focus:ring-2 focus:ring-teal-500 focus:border-teal-500 transition min-h-[120px]"
                  placeholder="شرح کامل موقعیت شغلی، مسئولیت‌ها و الزامات را وارد کنید..."
                  value={form.description}
                  onChange={(e) => updateField("description", e.target.value)}
                />
              </div>
            </div>
          )}

          {step === "review" && (
            <div className="space-y-6">
              <h2 className="text-2xl font-bold text-navy-800">بررسی نهایی</h2>
              <p className="text-gray-600">لطفاً اطلاعات وارد شده را بررسی کنید و سپس تأیید نمایید.</p>
              <div className="bg-gray-50 rounded-lg p-6 space-y-3">
                {[
                  { label: "نام شرکت", value: form.businessName },
                  { label: "صنعت", value: form.industry },
                  { label: "دپارتمان", value: form.department },
                  { label: "مکان", value: form.location },
                  { label: "سابقه مورد نیاز", value: `${form.experience} سال` },
                ].map((item) => (
                  <div key={item.label} className="flex justify-between py-2 border-b border-gray-200 last:border-0">
                    <span className="text-gray-500">{item.label}</span>
                    <span className="font-medium text-navy-800">{item.value || "—"}</span>
                  </div>
                ))}
                {form.description && (
                  <div className="pt-2">
                    <span className="text-gray-500 block mb-1">شرح موقعیت:</span>
                    <p className="text-sm text-gray-700 bg-white rounded p-3">{form.description}</p>
                  </div>
                )}
              </div>
            </div>
          )}

          {/* Navigation */}
          <div className="flex justify-between mt-8 pt-6 border-t border-gray-200">
            <GenButton
              variant="outline"
              onClick={() => {
                if (step === "business") return;
                setStep(steps[currentIdx - 1].id);
              }}
              disabled={step === "business"}
            >
              مرحله قبل
            </GenButton>

            {step !== "review" ? (
              <GenButton
                variant="primary"
                onClick={() => setStep(steps[currentIdx + 1].id)}
              >
                مرحله بعد
              </GenButton>
            ) : (
              <GenButton
                variant="primary"
                onClick={handleGenerate}
                loading={generating}
              >
                {generating ? "در حال تولید..." : "تولید موقعیت شغلی"}
              </GenButton>
            )}
          </div>
        </GenCard>

        {/* Footer info */}
        <div className="mt-8 text-center text-sm text-gray-400">
          <p>GenFlow v2 — معماری هیبریدی جزیره‌ای | قدرت گرفته از Rust + Remix</p>
        </div>
      </main>
    </div>
  );
}
